import * as anchor from "@coral-xyz/anchor";
import { Program} from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { ChronoToken } from "../target/types/chrono_token";
import idl from "../target/idl/chrono_token.json"

export class ChronoTokenSDK {
  program: Program<ChronoToken>;
  provider: anchor.AnchorProvider;

  constructor(provider: anchor.AnchorProvider, programId: PublicKey) {
    this.provider = provider;
    this.program = new Program<ChronoToken>(idl as ChronoToken, provider);
  }

  async initializeMint(
      decimals: number,
      supply: anchor.BN,
      freezeAuthority: PublicKey,
      enableChronoHook: boolean,
      chronoHookProgramId: PublicKey | null,
      equationType: any,
      pauseType: any,
      equationParams: any
  ): Promise<PublicKey> {
    const mint = anchor.web3.Keypair.generate();
    // @ts-ignore
    await this.program.methods.initializeMint(
        decimals,
        supply,
        freezeAuthority,
        0,
        enableChronoHook,
        chronoHookProgramId,
        equationType,
        pauseType,
        equationParams
    )
        .accounts({
          mint: mint.publicKey,
          authority: this.provider.wallet.publicKey,
          payer: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([mint])
        .rpc();

    return mint.publicKey;
  }

  async initializeTokenAccount(
      mint: PublicKey,
      owner: PublicKey,
      equationType: any
  ): Promise<PublicKey> {
    const tokenAccount = anchor.web3.Keypair.generate();
    const [decayPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("decay_pool"), tokenAccount.publicKey.toBuffer()],
        this.program.programId
    );

    await this.program.methods.initializeTokenAccount(null, equationType)
        .accounts({
          tokenAccount: tokenAccount.publicKey,
          decayPool,
          mint,
          authority: owner,
          payer: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([tokenAccount])
        .rpc();

    return tokenAccount.publicKey;
  }

  async transfer(
      fromPubkey: PublicKey,
      toPubkey: PublicKey,
      amount: anchor.BN
  ): Promise<string> {
    const mint = await this.program.account.tokenAccount.fetch(fromPubkey).then(acc => acc.mint);
    const [fromDecayPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("decay_pool"), fromPubkey.toBuffer()],
        this.program.programId
    );
    const [toDecayPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("decay_pool"), toPubkey.toBuffer()],
        this.program.programId
    );

    const tx = await this.program.methods.transfer(amount)
        .accounts({
          mint,
          from: fromPubkey,
          to: toPubkey,
          fromDecayPool,
          toDecayPool,
          authority: this.provider.wallet.publicKey,
        })
        .rpc();

    return tx;
  }

  // Add more methods for other instructions (mintTo, burn, pause, etc.)
}