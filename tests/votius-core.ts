import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { VotiusCore } from '../target/types/votius_core';
import { PublicKey } from '@solana/web3.js'
import { assert } from 'chai';

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.VotiusCore as Program<VotiusCore>;

describe('votius-core', () => {
  it('Initialize the experiment', async () => {
    const authority = provider.wallet.publicKey;
    const experimentId = new anchor.BN(1);

    const [experimentPda] = PublicKey.findProgramAddressSync([
      authority.toBuffer(),
      experimentId.toArrayLike(Buffer, 'le', 8)
    ], program.programId);

    await program.methods
      .initilizeExperiment(experimentId)
      .accounts({
        experiment: experimentPda,
        authority, systemProgram: anchor.web3.SystemProgram.programId
      } as any).rpc();

    const experimentAccount = await program.account.experiment.fetch(
      experimentPda
    );

    assert.equal(experimentAccount.experimentId.toNumber(), 1);
    assert.equal(experimentAccount.status.active != undefined, true)
    assert.ok(experimentAccount.authority.equals(authority));
    assert.ok(experimentAccount.createdAt.toNumber() > 0);

    const fakehash = Array.from(new Uint8Array(32).fill(1));
    const transaction_signature = await program.methods.recordEvent(fakehash).accounts({
      authority,
      experiment: experimentPda,
    } as any).rpc();

    const updatedExperiment =
      await program.account.experiment.fetch(experimentPda);

    console.log("Experiment ID:", updatedExperiment.experimentId.toNumber());
    console.log("Event Count:", updatedExperiment.eventCount.toNumber());
    console.log("Created At:", new Date(updatedExperiment.createdAt.toNumber() * 1000).toLocaleString());

    assert.equal(updatedExperiment.eventCount.toNumber(), 1);

    const attacker = anchor.web3.Keypair.generate();

    await provider.connection.requestAirdrop(attacker.publicKey, anchor.web3.LAMPORTS_PER_SOL);

    let failed = false;

    try {
      await program.methods.recordEvent(fakehash).accounts({
        experiment: experimentPda,
        authority: attacker.publicKey,
      } as any).signers([attacker]).rpc();
      console.log("CRITICAL: Attacker successfully bypassed security!");
    } catch (error: any) {
      failed = true;
      assert.include(error.toString(), "Unauthorized");
    }
    assert.isTrue(failed, "Unauthorized write should failed");

    const transactionData = await program.provider.connection.getTransaction(transaction_signature, { commitment: "confirmed" });

    let logs = transactionData.meta.logMessages || [];
    console.log("Transaction Logs");
    logs.forEach((log) => console.log(log))

    await program.methods
      .completeExperiment()
      .accounts({
        experiment: experimentPda,
        authority,
      })
      .rpc();

    const completedExperiment =
      await program.account.experiment.fetch(experimentPda);

    assert.equal(
      completedExperiment.status.completed !== undefined,
      true
    );

    failed = false;

    try {
      await program.methods
        .completeExperiment()
        .accounts({
          experiment: experimentPda,
          authority,
        })
        .rpc();
    } catch (err) {
      failed = true;
    }

    assert.isTrue(failed, "Experiment should not complete twice");


    const postTxSig = await program.methods
      .recordEvent(Array.from(fakehash))
      .accounts({
        experiment: experimentPda,
        authority,
      } as any)
      .rpc();

    const finalExperiment =
      await program.account.experiment.fetch(experimentPda);

    assert.equal(finalExperiment.eventCount.toNumber(), 2);

    const postTx = await provider.connection.getTransaction(postTxSig, {
      commitment: "confirmed",
    });

    logs = postTx?.meta?.logMessages || [];
    console.log("Post-completion transaction logs:");
    logs.forEach((log) => console.log(log));

  });

})