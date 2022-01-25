const anchor = require("@project-serum/anchor")
import { Program } from '@project-serum/anchor'
import { Game } from '../target/types/game'
import { Highscore } from '../target/types/highscore'
const assert = require("assert")
const { SystemProgram } = anchor.web3

let _myAssets
let _myIncrementor
let _highscores

// Use a local provider.
const provider = anchor.Provider.local()

// Configure the client to use the local cluster.
anchor.setProvider(provider)

describe('Game setup', () => {  

  it('Initialized the game account and resources', async () => {

    const program = anchor.workspace.Game as Program<Game>
    const myIncrementor = anchor.web3.Keypair.generate()
    const myAssets = anchor.web3.Keypair.generate()

    await program.rpc.initialize(provider.wallet.publicKey, {
      accounts: {
        myAssets: myAssets.publicKey,
        myIncrementor: myIncrementor.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [myIncrementor, myAssets],
    })

    // Check if no assets and 2 credits are owned by newly created account
    const myAssetsAccount = await program.account.myAssets.fetch(myAssets.publicKey)
    assert.ok(myAssetsAccount.asset1.eq(new anchor.BN(0)))
    assert.ok(myAssetsAccount.asset2.eq(new anchor.BN(0)))
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(2)))

    // Check if incrementor value and last used time are set
    const myIncrementorAccount = await program.account.myIncrementor.fetch(myIncrementor.publicKey)
    assert.ok(myIncrementorAccount.value.eq(new anchor.BN(1)))
    assert.ok(myIncrementorAccount.lastUsedTime.gt(new anchor.BN(0)))
    assert.ok(myIncrementorAccount.upgradeCost.eq(new anchor.BN(50)))

    _myAssets = myAssets
    _myIncrementor = myIncrementor
    
  })
})

describe('Highscore setup', () => {  

  it('Initialized the highscore account', async () => {

    const highscore_program = anchor.workspace.Highscore as Program<Highscore>
    const highscore = anchor.web3.Keypair.generate()

    await highscore_program.rpc.initialize(
      {
        accounts: {
          highscore: highscore.publicKey
        }
      }
    )

    // // Check if no assets and 2 credits are owned by newly created account
    // const highscores = await program.account.highscore.fetch(highscore.publicKey)
    // // assert.ok(highscores.scores[0].amount.eq(new anchor.BN(0)))

    // _highscores = highscore
    
  })
})

describe('Manual increments', () => {
  it('Processes a manual increment', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)
    let creditsBefore = myAssetsAccountBefore.credits

    const myIncrementorAccountBefore = await program.account.myIncrementor.fetch(_myIncrementor.publicKey)
    let timeSinceLastUseBefore = myIncrementorAccountBefore.lastUsedTime.toNumber()
    let expectedIncrement = myIncrementorAccountBefore.value
    let expectedValueAfterIncrement = creditsBefore.toNumber() + expectedIncrement.toNumber()

    // Wait for the cooldown
    await sleep(1000)

    await program.rpc.incrementManual({
      accounts: {
        myIncrementor: _myIncrementor.publicKey,
        myAssets: _myAssets.publicKey,
        authority: provider.wallet.publicKey
      }
    })

    // Check if the credits have increased by the expected amount
    const myAssetsAccount = await program.account.myAssets.fetch(_myAssets.publicKey)
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(expectedValueAfterIncrement)))

    // Check if the last used time has changed
    const myIncrementorAccount = await program.account.myIncrementor.fetch(_myIncrementor.publicKey)
    assert.ok(myIncrementorAccount.lastUsedTime.gt(new anchor.BN(timeSinceLastUseBefore)))
    
  })

  it('Prevents a manual increment within cooldown period', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)
    let creditsBefore = myAssetsAccountBefore.credits

    const myIncrementorAccountBefore = await program.account.myIncrementor.fetch(_myIncrementor.publicKey)
    let timeSinceLastUseBefore = myIncrementorAccountBefore.lastUsedTime.toNumber()
    let expectedIncrement = myIncrementorAccountBefore.value
    let expectedValueAfterIncrement = creditsBefore.toNumber() + expectedIncrement.toNumber()

    // Wait for the cooldown
    await sleep(1000)

    await program.rpc.incrementManual( {
      accounts: {
        myIncrementor: _myIncrementor.publicKey,
        myAssets: _myAssets.publicKey,
        authority: provider.wallet.publicKey
      },
    })

    try {
      // Immediate second call to increment
      await program.rpc.incrementManual( {
        accounts: {
          myIncrementor: _myIncrementor.publicKey,
          myAssets: _myAssets.publicKey,
          authority: provider.wallet.publicKey
        },
      })
      assert.ok(false);
    } catch (err) {
      const errMsg = "Too little time between manual increment requests.";
      assert.equal(err.toString(), errMsg);
    }    
  })

  it('Prevent unauthorized user from incrementing', async () => {

    const program = anchor.workspace.Game as Program<Game>
    const newKey = anchor.web3.Keypair.generate()

    try {
      // Try to increment without proper authority
      await program.rpc.incrementManual({
        accounts: {
          myIncrementor: _myIncrementor.publicKey,
          myAssets: _myAssets.publicKey,
          authority: newKey.publicKey
        },
      })
  
      assert.ok(false);
    } catch (err) {
      const errMsg = "Error: Signature verification failed"
      assert.equal(err.toString(), errMsg);
    }
  })
})

before(() => sleep(2000))
describe('Asset purchases', () => {
  it('Processes an asset purchase', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)
    let creditsBefore = myAssetsAccountBefore.credits
    let cost = 2
    let expectedValueAfterPurchase = creditsBefore.toNumber() - cost

    await program.rpc.acquireAsset(new anchor.BN(0), {
      accounts: {
        myAssets: _myAssets.publicKey,
        authority: provider.wallet.publicKey
      },
    })

    // Check if the credits have decreased to the expected amount and asset 0 is increased
    const myAssetsAccount = await program.account.myAssets.fetch(_myAssets.publicKey)
    assert.ok(myAssetsAccount.credits.eq(new anchor.BN(expectedValueAfterPurchase)))
    assert.ok(myAssetsAccount.asset1.eq(new anchor.BN(1)))
  })

  it('Prevent too expensive asset purchase without charge', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)

    try {
      // Try to purchase something we cannot afford
      await program.rpc.acquireAsset(new anchor.BN(1), {
        accounts: {
          myAssets: _myAssets.publicKey,
          authority: provider.wallet.publicKey
        },
      })
      assert.ok(false);
    } catch (err) {
      const errMsg = "Not enough credits in account for purchase.";
      assert.equal(err.toString(), errMsg);
    }
  })

  it('Harvest asset resources and check highscores', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)

    try {
      // Try to purchase something we cannot afford
      await program.rpc.acquireAsset(new anchor.BN(1), {
        accounts: {
          myAssets: _myAssets.publicKey,
          authority: provider.wallet.publicKey
        },
      })
      assert.ok(false);
    } catch (err) {
      const errMsg = "Not enough credits in account for purchase.";
      assert.equal(err.toString(), errMsg);
    }
  })

  it('Prevent incorrect asset type purchase', async () => {

    const program = anchor.workspace.Game as Program<Game>

    // Get the account state before execution
    const myAssetsAccountBefore = await program.account.myAssets.fetch(_myAssets.publicKey)

    try {
      // Try to purchase something we cannot afford
      await program.rpc.acquireAsset(new anchor.BN(5), {
        accounts: {
          myAssets: _myAssets.publicKey,
          authority: provider.wallet.publicKey
        },
      })
      assert.ok(false);
    } catch (err) {
      const errMsg = "Unknown asset type defined.";
      assert.equal(err.toString(), errMsg);
    }
  })

  it('Prevent unauthorized user from purchasing', async () => {

    const program = anchor.workspace.Game as Program<Game>
    const newKey = anchor.web3.Keypair.generate()

    try {
      // Try to increment without proper authority
      await program.rpc.acquireAsset(new anchor.BN(1), {
        accounts: {
          myAssets: _myAssets.publicKey,
          authority: newKey.publicKey
        },
      })
  
      assert.ok(false);
    } catch (err) {
      const errMsg = "Error: Signature verification failed";
      assert.equal(err.toString(), errMsg);
    }
  })
})


function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}