import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenTimeLockAgreement } from "../target/types/token_time_lock_agreement";

module.exports = async function (provider: anchor.AnchorProvider) {
  anchor.setProvider(provider);

  const program = anchor.workspace.tokenTimeLockAgreement as Program<TokenTimeLockAgreement>;

  const [settings] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("settings")],
    program.programId
  );

  const tx = await program.methods.initialize({
    cancelTimeout: new anchor.BN(60 * 60 * 72), // 72 hours
    commissionBasisPoints: 100, // 1%
    owner: new anchor.web3.PublicKey("DFbHUAt744X6K37nygXyXjqEw9K6qAKAHDZNzeY3BP7V"), // program.provider.wallet.publicKey,
  }).accountsStrict({
    settings: settings,
    payer: program.provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId
  }).rpc({
    skipPreflight: true // for some reason, this is required
  });

  console.log("Your transaction signature", tx);
};
