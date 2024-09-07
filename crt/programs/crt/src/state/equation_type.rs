use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum ChronoEquationType {
    Subscription,
    Inflationary,
    Deflationary,
    Linear,
    Exponential,
}

impl ChronoEquationType {
    pub fn get_equation(&self, x: u64, t: i64, params: &EquationParams) -> Result<String> {
        let base_equation = match self {
            ChronoEquationType::Subscription => {
                let expiration_time = params.expiration_time.ok_or(ProgramError::InvalidAccountData)?;
                format!("{} * (({}) <= {} ? 1 : 0)", x, t, expiration_time)
            }
            ChronoEquationType::Inflationary => {
                let snapshot_time = params.snapshot_time.ok_or(ProgramError::InvalidAccountData)?;
                let inflation_rate = params.inflation_rate.ok_or(ProgramError::InvalidAccountData)?;
                let time_unit = params.time_unit.ok_or(ProgramError::InvalidAccountData)?;
                format!("{} + (({}) - {}) * {} / {}", x, t, snapshot_time, inflation_rate, time_unit)
            }
            ChronoEquationType::Deflationary => {
                let snapshot_time = params.snapshot_time.ok_or(ProgramError::InvalidAccountData)?;
                let decay_rate = params.decay_rate.ok_or(ProgramError::InvalidAccountData)?;
                let time_unit = params.time_unit.ok_or(ProgramError::InvalidAccountData)?;
                format!("max(0, {} - (({}) - {}) * {} / {})", x, t, snapshot_time, decay_rate, time_unit)
            }
            ChronoEquationType::Linear => {
                let snapshot_time = params.snapshot_time.ok_or(ProgramError::InvalidAccountData)?;
                let slope = params.slope.ok_or(ProgramError::InvalidAccountData)?;
                format!("{} + (({}) - {}) * {}", x, t, snapshot_time, slope)
            }
            ChronoEquationType::Exponential => {
                let snapshot_time = params.snapshot_time.ok_or(ProgramError::InvalidAccountData)?;
                let decay_constant = params.decay_constant.ok_or(ProgramError::InvalidAccountData)?;
                let time_unit = params.time_unit.ok_or(ProgramError::InvalidAccountData)?;
                format!("{} * exp(-{} * (({}) - {}) / {})", x, decay_constant, t, snapshot_time, time_unit)
            }
            // ChronoEquationType::Paused => {
            //     "x".to_string()
            // }
        };

        // Apply ReUp boost if it exists
        if let Some(reup_boost) = params.reup_boost {
            Ok(format!("({}) + {}", base_equation, reup_boost))
        } else {
            Ok(base_equation)
        }
    }

    pub fn get_params(&self) -> EquationParams {
        match self {
            ChronoEquationType::Subscription => EquationParams {
                expiration_time: Some(0), // You'll need to set this appropriately
                ..Default::default()
            },
            ChronoEquationType::Inflationary => EquationParams {
                inflation_rate: Some(0), // Set this appropriately
                time_unit: Some(86400), // Assuming daily inflation
                ..Default::default()
            },
            ChronoEquationType::Deflationary => EquationParams {
                decay_rate: Some(0), // Set this appropriately
                time_unit: Some(86400), // Assuming daily decay
                ..Default::default()
            },
            ChronoEquationType::Linear => EquationParams {
                slope: Some(0), // Set this appropriately
                ..Default::default()
            },
            ChronoEquationType::Exponential => EquationParams {
                decay_constant: Some(0 as f64), // Set this appropriately
                time_unit: Some(86400), // Assuming daily decay
                ..Default::default()
            },
        }
    }

    pub fn get_equation_string(&self) -> String {
        match self {
            ChronoEquationType::Subscription => "x * (t <= expiration_time ? 1 : 0)".to_string(),
            ChronoEquationType::Inflationary => "x + (t * inflation_rate / time_unit)".to_string(),
            ChronoEquationType::Deflationary => "max(0, x - (t * decay_rate / time_unit))".to_string(),
            ChronoEquationType::Linear => "x + (t * slope)".to_string(),
            ChronoEquationType::Exponential => "x * exp(-decay_constant * t / time_unit)".to_string(),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default)]
pub struct EquationParams {
    pub snapshot_time: Option<i64>,
    pub expiration_time: Option<i64>,
    pub inflation_rate: Option<u64>,
    pub decay_rate: Option<u64>,
    pub time_unit: Option<u64>,
    pub slope: Option<i64>,
    pub decay_constant: Option<f64>,
    pub reup_boost: Option<u64>,
}

impl EquationParams {
    pub fn new() -> Self {
        Self {
            snapshot_time: None,
            expiration_time: None,
            inflation_rate: None,
            decay_rate: None,
            time_unit: None,
            slope: None,
            decay_constant: None,
            reup_boost: None,
        }
    }
}