import * as anchor from "@coral-xyz/anchor";
import { Program} from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { ChronoToken } from "../target/types/chrono_token";
import idl from "../target/idl/chrono_token.json"

// Define the custom types as they are in your IDL
type EquationType = 
  | { subscription: {} }
  | { inflationary: {} }
  | { deflationary: {} }
  | { linear: {} }
  | { exponential: {} };

type PauseType = 
  | { pause: {} }
  | { reUp: {} }


// EquationParams type with all properties as required
type EquationParams = {
  snapshotTime: anchor.BN;
  expirationTime: anchor.BN;
  inflationRate: anchor.BN;
  decayRate: anchor.BN;
  timeUnit: anchor.BN;
  slope: anchor.BN;
  decayConstant: number;
  reupBoost: anchor.BN;
};

///this the typescript version of the eqaution calculation should ultimately be in a different file
interface TokenAccountData {
  mint: PublicKey;
  owner: PublicKey;
  lastBalanceSnapshot: anchor.BN;
  currentChronoEquation: EquationType;
  creationTime: anchor.BN;
  state: { initialized: {} } | { frozen: {} } | { revoked: {} };
  delegate: PublicKey;
  delegatedAmount: anchor.BN;
  closeAuthority: PublicKey | null;
}

interface ChronoExtension {
  authority: PublicKey;
  programId: PublicKey;
  equationType: EquationType;
  pauseType: PauseType;
  equationParams: EquationParams;
  reupPercentage: number;
}

export class ChronoTokenSDK {
  program: Program<ChronoToken>;
  provider: anchor.AnchorProvider;

  constructor(provider: anchor.AnchorProvider, programId: PublicKey) {
    this.provider = provider;
    this.program = new Program<ChronoToken>(idl as ChronoToken, provider);
  }

  
  private calculateBalance(
    lastBalanceSnapshot: anchor.BN,
    equation: EquationType,
    params: EquationParams,
    currentTime: anchor.BN
  ): anchor.BN {
    const elapsedTime = currentTime.sub(params.snapshotTime);

    if ('subscription' in equation) {
      return currentTime.lte(params.expirationTime) ? lastBalanceSnapshot : new anchor.BN(0);
    } else if ('inflationary' in equation) {
      const inflationAmount = elapsedTime.mul(params.inflationRate).div(params.timeUnit);
      return lastBalanceSnapshot.add(inflationAmount);
    } else if ('deflationary' in equation) {
      const decayAmount = elapsedTime.mul(params.decayRate).div(params.timeUnit);
      return anchor.BN.max(new anchor.BN(0), lastBalanceSnapshot.sub(decayAmount));
    } else if ('linear' in equation) {
      return lastBalanceSnapshot.add(elapsedTime.mul(params.slope));
    } else if ('exponential' in equation) {
      // Note: This is a simplified calculation and may not be precise for large numbers
      const decayFactor = Math.exp(-params.decayConstant * elapsedTime.toNumber() / params.timeUnit.toNumber());
      return lastBalanceSnapshot.muln(decayFactor);
    } else {
      throw new Error('Unknown equation type');
    }
  }

  ///To do: update initMint to use extensions account and in turn, create get balance with an account structure rather than data
  
   async initializeMint(
    mint: PublicKey,
    authority: Keypair,
    decimals: number,
    supply: anchor.BN,
    freezeAuthority: PublicKey | null,
    enableChronoHook: boolean,
    chronoHookProgramId: PublicKey | null,
    equationType: EquationType | null,
    pauseType: PauseType | null,
    equationParams: EquationParams | null,
    reupPercentage: number | null
  ): Promise<string> {
    
    const [ chronoExtensionAccount ] = PublicKey.findProgramAddressSync(
      [Buffer.from('chrono_extension'), mint.toBuffer()],
      this.program.programId
    );

    let accounts = {
      mint: mint,
      chronoExtension: chronoExtensionAccount,
      authority: authority.publicKey,
      payer: this.provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }

    const tx = await this.program.methods
      .initializeMint(
        decimals,
        supply,
        freezeAuthority,
        0, // bump (you might want to calculate this)
        enableChronoHook,
        chronoHookProgramId,
        equationType,
        pauseType,
        equationParams,
        reupPercentage
      )
      .accounts(accounts)
      .signers([authority])
      .rpc();

    return tx;
  }

  async initializeTokenAccount(
    mint: PublicKey,
    ///get associated token acount
    tokenAccount: PublicKey,
    authority: Keypair,
    payer: PublicKey,
    delegate?: PublicKey
  ): Promise<{ tokenAccount: PublicKey; decayPool: PublicKey; txSignature: string }> {


    // Derive the PDA for the decay pool
    const [decayPool] = PublicKey.findProgramAddressSync(
      [Buffer.from("decay_pool"), tokenAccount.toBuffer()],
      this.program.programId
    );

    let accounts = {
      tokenAccount: tokenAccount,
      decayPool: decayPool,
      mint: mint,
      authority: authority.publicKey,
      payer: payer,
      systemProgram: SystemProgram.programId,
    }

    const tx = await this.program.methods
      .initializeTokenAccount(delegate ? new PublicKey(delegate) : null)
      .accounts(accounts)
      .signers([authority])
      .rpc();

    return {
      tokenAccount: tokenAccount,
      decayPool: decayPool,
      txSignature: tx,
    };
  }


  async transfer(
      mint: PublicKey,
      fromPubkey: PublicKey,
      toPubkey: PublicKey,
      amount: anchor.BN
  ): Promise<string> {
    

    const [fromDecayPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("decay_pool"), fromPubkey.toBuffer()],
        this.program.programId
    );
    const [toDecayPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("decay_pool"), toPubkey.toBuffer()],
        this.program.programId
    );

    //Declared the accounts object separately because object literal had a type issue
    let accounts = {
      mint,
      from: fromPubkey,
      to: toPubkey,
      fromDecayPool,
      toDecayPool,
      authority: this.provider.wallet.publicKey,
    }

    const tx = await this.program.methods
        .transfer(amount)
        .accounts(accounts)
        .rpc();

    return tx;
  }

  async mintTo(
    mint: PublicKey,
    tokenAccount: PublicKey,
    amount: anchor.BN
  ): Promise<string> {


    const tx = await this.program.methods
      .mintTo(amount)
      .accounts({
        mint: mint,
        tokenAccount: tokenAccount,
        authority: this.provider.wallet.publicKey,
      })
      .rpc();

    return tx;
  }

  async reup(
    mint: PublicKey,
    tokenAccount: PublicKey,
    decayPool: PublicKey,
    chronoHookProgram: PublicKey
  ): Promise<string> {
    const tx = await this.program.methods
      .reup()
      .accounts({
        mint: mint,
        tokenAccount: tokenAccount,
        decayPool: decayPool,
        authority: this.provider.wallet.publicKey,
        chronoHookProgram: chronoHookProgram,
      })
      .rpc();

    return tx;
  }

  async pauseDecay(
    mint: PublicKey,
    tokenAccount: PublicKey,
    chronoHookProgram: PublicKey
  ): Promise<string> {
    const tx = await this.program.methods
      .pauseDecay()
      .accounts({
        mint: mint,
        tokenAccount: tokenAccount,
        authority: this.provider.wallet.publicKey,
        chronoHookProgram: chronoHookProgram,
      })
      .rpc();

    return tx;
  }


  async burn(
    mint: PublicKey,
    tokenAccount: PublicKey,
    amount: anchor.BN
  ): Promise<string> {


    const tx = await this.program.methods
      .burn(amount)
      .accounts({
        mint: mint,
        tokenAccount: tokenAccount,
        authority: this.provider.wallet.publicKey,
      })
      .rpc();

    return tx;
  }

  ///get balance
  async getTokenAccountBalance(tokenAccountAddress: PublicKey): Promise<anchor.BN> {
    const tokenAccount = await this.program.account.tokenAccount.fetch(tokenAccountAddress);
    const currentTime = new anchor.BN(Math.floor(Date.now() / 1000));

    // Fetch the mint account
    const mint = await this.program.account.mint.fetch(tokenAccount.mint);

    // Get the ChronoExtension PDA
    const [chronoExtensionPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('chrono_extension'), tokenAccount.mint.toBuffer()],
      this.program.programId
    );

    // Fetch the ChronoExtension account
    let chronoExtension: ChronoExtension | null = null;
    try {
      chronoExtension = await this.program.account.chronoExtension.fetch(chronoExtensionPDA);
    } catch (error) {
      console.error('Failed to fetch ChronoExtension:', error);
      // If there's no chrono extension, return the last balance snapshot
      return tokenAccount.lastBalanceSnapshot;
    }

    if (!chronoExtension) {
      // If there's no chrono extension, return the last balance snapshot
      return tokenAccount.lastBalanceSnapshot;
    }

    return this.calculateBalance(
      tokenAccount.lastBalanceSnapshot,
      tokenAccount.currentChronoEquation,
      chronoExtension.equationParams,
      currentTime
    );
  }
}
