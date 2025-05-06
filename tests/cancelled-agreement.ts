import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenTimeLockAgreement } from "../target/types/token_time_lock_agreement";
import { TOKEN_PROGRAM_ID, NATIVE_MINT, createAssociatedTokenAccount, createSyncNativeInstruction, getOrCreateAssociatedTokenAccount, transfer } from '@solana/spl-token';
import { assert } from "chai";

describe("cancelled-agreement", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.tokenTimeLockAgreement as Program<TokenTimeLockAgreement>;

  const companyWallet = anchor.web3.Keypair.generate();
  const tokenSeller = anchor.web3.Keypair.generate();
  const otherWallet = anchor.web3.Keypair.generate();
  const investors = [
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
  ];

  const agreementId = new anchor.BN(2);
  const expectedPayment = new anchor.BN(600);
  const expectedTokens = new anchor.BN(100);
  const holdDuration = new anchor.BN(1);

  const paymentTokenMint = NATIVE_MINT;
  const projectTokenMint = NATIVE_MINT;
  const recipientWallet = anchor.web3.PublicKey.unique();
  const agreement = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("agreement"), Uint8Array.from(agreementId.toBuffer('le', 8))],
    program.programId
  )[0];
  const paymentTokenAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("payment"), Uint8Array.from(agreementId.toBuffer('le', 8))],
    program.programId
  )[0];
  const projectTokenAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("project"), Uint8Array.from(agreementId.toBuffer('le', 8))],
    program.programId
  )[0];
  const settings = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("settings")],
    program.programId
  )[0];

  const allocations = [
    {
      amount: new anchor.BN(100),
      tokenAmount: new anchor.BN(20),
      wallet: investors[0].publicKey
    },
    {
      amount: new anchor.BN(200),
      tokenAmount: new anchor.BN(30),
      wallet: investors[1].publicKey
    },
    {
      amount: new anchor.BN(300),
      tokenAmount: new anchor.BN(50),
      wallet: investors[2].publicKey
    }
  ]

  let investorTokenAccounts: anchor.web3.PublicKey[] = [];
  let sellerTokenAccount: anchor.web3.PublicKey;
  let otherTokenAccount: anchor.web3.PublicKey;
  let ownerTokenAccount: anchor.web3.PublicKey;

  async function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  before(async () => {
    // airdrop some SOL to investors
    for (const investor of investors) {
      await program.provider.connection.requestAirdrop(investor.publicKey, 1000000000);

      const investorTokenAccount = await createAssociatedTokenAccount(
        program.provider.connection,
        program.provider.wallet.payer,
        NATIVE_MINT,
        investor.publicKey
      );
      investorTokenAccounts.push(investorTokenAccount);

      const wrapTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.wallet.publicKey,
          toPubkey: investorTokenAccount,
          lamports: 1_000,
        }),
        createSyncNativeInstruction(investorTokenAccount)
      );

      const txSignature = await program.provider.sendAndConfirm(wrapTx);
      console.log('Wrap TX:', txSignature);
    }

    // airdrop some SOL to token seller and other wallet
    await program.provider.connection.requestAirdrop(otherWallet.publicKey, 1000000000);
    await program.provider.connection.requestAirdrop(tokenSeller.publicKey, 1000000000);

    otherTokenAccount = (await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      program.provider.wallet.payer,
      NATIVE_MINT,
      otherWallet.publicKey
    )).address;
    ownerTokenAccount = (await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      program.provider.wallet.payer,
      NATIVE_MINT,
      program.provider.wallet.publicKey
    )).address;
    sellerTokenAccount = (await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      program.provider.wallet.payer,
      NATIVE_MINT,
      tokenSeller.publicKey
    )).address;

    const wrapTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: program.provider.wallet.publicKey,
        toPubkey: otherTokenAccount,
        lamports: 1_000,
      }),
      anchor.web3.SystemProgram.transfer({
        fromPubkey: program.provider.wallet.publicKey,
        toPubkey: sellerTokenAccount,
        lamports: 1_000,
      }),
      createSyncNativeInstruction(otherTokenAccount),
      createSyncNativeInstruction(sellerTokenAccount)
    );

    const txSignature = await program.provider.sendAndConfirm(wrapTx);
    console.log('Wrap TX:', txSignature);
  });

  it("Initializes the contract", async () => {
    let settingsAccount = await program.account.settings.fetchNullable(settings);
    if (settingsAccount == null) {
      const tx = await program.methods.initialize({
        commissionBasisPoints: new anchor.BN(100), // 1%
        cancelTimeout: new anchor.BN(1),
        owner: program.provider.wallet.publicKey,
      }).accounts({
        settings,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();

      console.log("Your transaction signature", tx);
    } else {
      console.warn("Settings already initialized");
    }
  });

  it("Initializes the agreement", async () => {
    const tx = await program.methods.initializeAgreement({
      agreementId,
      expectedPayment,
      expectedTokens,
      holdDuration,
    }).accounts({
      settings,
      agreement,
      paymentTokenMint,
      projectTokenMint,
      paymentTokenAccount,
      projectTokenAccount,
      companyWallet: companyWallet.publicKey,
      recipientWallet,
      tokenSeller: tokenSeller.publicKey,
      payer: program.provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Adds investors", async () => {
    const tx = await program.methods.addInvestors({ allocations }).accounts({
      agreement,
      payer: program.provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).remainingAccounts(investors.map(investor => {
      return {
        pubkey: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("investor"), agreement.toBuffer(), investor.publicKey.toBuffer()],
          program.programId
        )[0],
        isWritable: true,
        isSigner: false
      }
    })).rpc();
    console.log("Your transaction signature", tx);

    // check investors
    for (const allocation of allocations) {
      const investor = await program.account.investor.fetch(
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("investor"), agreement.toBuffer(), allocation.wallet.toBuffer()],
          program.programId
        )[0]
      );

      assert.equal(investor.agreement.toBase58(), agreement.toBase58());
    }
  });

  it("Deposits tokens by seller", async () => {
    await transfer(program.provider.connection, tokenSeller, sellerTokenAccount, projectTokenAccount, tokenSeller.publicKey, expectedTokens.toNumber());

    const tx = await program.methods.processTokenDeposit({}).accounts({
      agreement,
      agreementTokenAccount: projectTokenAccount,
      payer: tokenSeller.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    }).signers([tokenSeller]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposits stablecoins by part of investors", async () => {
    for (var i = 0; i < investors.length - 1; i++) {
      const allocation = allocations[i];
      const investor = investors[i];
      const investorTokenAccount = investorTokenAccounts[i];

      const tx = await program.methods.depositStablecoins({ amount: allocation.amount }).accounts({
        agreement,
        investor: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("investor"), agreement.toBuffer(), investor.publicKey.toBuffer()],
          program.programId
        )[0],
        destinationTokenAccount: paymentTokenAccount,
        payerTokenAccount: investorTokenAccount,
        payer: investor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      }).signers([investor]).rpc();

      console.log("Your transaction signature", tx);
    }
  });

  it("Cancel agreement", async () => {
    await sleep(1000);

    const tx = await program.methods.cancelAgreement({}).accounts({
      agreement,
      projectTokenAccount,
      investor: null,
      payer: program.provider.wallet.publicKey,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    }).rpc();

    console.log("Your transaction signature", tx);
  });

  it("Withdraws cancelled funds by investors", async () => {
    for (var i = 0; i < investors.length - 1; i++) {
      const investor = investors[i];
      const investorTokenAccount = investorTokenAccounts[i];

      const tx = await program.methods.withdrawCancelledFunds({}).accounts({
        agreement,
        investor: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("investor"), agreement.toBuffer(), investor.publicKey.toBuffer()],
          program.programId
        )[0],
        agreementTokenAccount: paymentTokenAccount,
        destinationTokenAccount: investorTokenAccount,
        payer: investor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      }).signers([investor]).rpc();

      console.log("Your transaction signature", tx);
    }
  });

  it("Withdraw cancelled funds by token seller", async () => {
    const tx = await program.methods.withdrawCancelledFunds({}).accounts({
      agreement,
      investor: null,
      agreementTokenAccount: projectTokenAccount,
      destinationTokenAccount: sellerTokenAccount,
      payer: tokenSeller.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    }).signers([tokenSeller]).rpc();

    console.log("Your transaction signature", tx);
  });

});
