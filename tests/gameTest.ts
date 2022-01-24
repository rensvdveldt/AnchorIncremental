const anchor = require("@project-serum/anchor");
import { Program } from '@project-serum/anchor';
import { Game } from '../target/types/game';
const assert = require("assert");
const { SystemProgram } = anchor.web3;

let _myAssets;
let _myIncrementor;

describe('Game setup', () => {

  // Use a local provider.
  const provider = anchor.Provider.env()

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it('Initialized the game account and resources', async () => {

    const program = anchor.workspace.Game as Program<Game>;
    const myIncrementor = anchor.web3.Keypair.generate();
    const myAssets = anchor.web3.Keypair.generate();

    await program.rpc.initialize( {
      accounts: {
        myAssets: myAssets.publicKey,
        myIncrementor: myIncrementor.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [myIncrementor, myAssets],
    });

    // Check if no assets and 2 credits are owned by newly created account
    const myAssetsAccount = await program.account.myAssets.fetch(myAssets.publicKey);
    assert.ok(myAssetsAccount.asset1.eq(new anchor.BN(0)));
    assert.ok(myAssetsAccount.asset2.eq(new anchor.BN(0)));
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(2)));

    // Check if incrementor value and last used time are set
    const myIncrementorAccount = await program.account.myIncrementor.fetch(myIncrementor.publicKey);
    assert.ok(myIncrementorAccount.value.eq(new anchor.BN(1)));
    assert.ok(myIncrementorAccount.lastUsedTime.gt(new anchor.BN(0)));
    assert.ok(myIncrementorAccount.upgradeCost.eq(new anchor.BN(50)));

    _myAssets = myAssets;
    _myIncrementor = myIncrementor;
    
  });

  
});

describe('Manual increments', () => {
  it('Processes a manual increment', async () => {

    const program = anchor.workspace.Game as Program<Game>;

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey);
    let creditsBefore = myAssetsAccountBefore.credits;

    const myIncrementorAccountBefore = await program.account.myIncrementor.fetch(_myIncrementor.publicKey);
    let timeSinceLastUseBefore = myIncrementorAccountBefore.lastUsedTime.toNumber();
    let expectedIncrement = myIncrementorAccountBefore.value;
    let expectedValueAfterIncrement = creditsBefore.toNumber() + expectedIncrement.toNumber();

    // Wait for the cooldown
    await sleep(1000);

    await program.rpc.incrementManual( {
      accounts: {
        myIncrementor: _myIncrementor.publicKey,
        myAssets: _myAssets.publicKey,
      },
      // signers: [_myIncrementor, _myAssets],
    });

    // Check if the credits have increased by the expected amount
    const myAssetsAccount = await program.account.myAssets.fetch(_myAssets.publicKey);
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(expectedValueAfterIncrement)));

    // Check if the last used time has changed
    const myIncrementorAccount = await program.account.myIncrementor.fetch(_myIncrementor.publicKey);
    assert.ok(myIncrementorAccount.lastUsedTime.gt(new anchor.BN(timeSinceLastUseBefore)));
    
  });

  it('Prevents a manual increment within cooldown period', async () => {

    const program = anchor.workspace.Game as Program<Game>;

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey);
    let creditsBefore = myAssetsAccountBefore.credits;

    const myIncrementorAccountBefore = await program.account.myIncrementor.fetch(_myIncrementor.publicKey);
    let timeSinceLastUseBefore = myIncrementorAccountBefore.lastUsedTime.toNumber();
    let expectedIncrement = myIncrementorAccountBefore.value;
    let expectedValueAfterIncrement = creditsBefore.toNumber() + expectedIncrement.toNumber();

    // Wait for the cooldown
    await sleep(1000);

    await program.rpc.incrementManual( {
      accounts: {
        myIncrementor: _myIncrementor.publicKey,
        myAssets: _myAssets.publicKey,
      },
      // signers: [_myIncrementor, _myAssets],
    });

    // Second call to increment
    await program.rpc.incrementManual( {
      accounts: {
        myIncrementor: _myIncrementor.publicKey,
        myAssets: _myAssets.publicKey,
      },
      // signers: [_myIncrementor, _myAssets],
    });

    // Check if the credits have increased by the expected amount
    const myAssetsAccount = await program.account.myAssets.fetch(_myAssets.publicKey);
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(expectedValueAfterIncrement)));

    // Check if the last used time has changed
    const myIncrementorAccount = await program.account.myIncrementor.fetch(_myIncrementor.publicKey);
    assert.ok(myIncrementorAccount.lastUsedTime.gt(new anchor.BN(timeSinceLastUseBefore)));
    
  });
});



function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}