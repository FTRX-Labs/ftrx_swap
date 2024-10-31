import * as anchor from "@coral-xyz/anchor";

import { PublicKey } from "@solana/web3.js";
import { FtrxSwap } from "../target/types/ftrx_swap";
import { TestValues, createValues, expectRevert,mintingTokens } from "./utils";
import { expect } from "chai";
import { Program, BN, web3  } from "@coral-xyz/anchor";
import { superUserKey } from "./testKeys";

import {
  getAccount,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  createSyncNativeInstruction,
  createCloseAccountInstruction,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  getMint,
} from "@solana/spl-token";

describe("ftrx_swap", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FtrxSwap as Program<FtrxSwap>;
  const connection = program.provider.connection;


  async function get_onchain_logs(connection,tx1){
    
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const txDetails = await connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("SWAP DETAILS",txDetails.meta.logMessages)
  }


  //Creating mints
  let values: TestValues;
  values = createValues();
  // Setting the LP fee for the new pool
  let new_pool_lp_fee_in_bp=10
  const lpFeeBuffer = Buffer.alloc(2) // 2 bytes for u16
  lpFeeBuffer.writeUInt16LE(new_pool_lp_fee_in_bp) 

  //Creating the PDAs
  const superUser = superUserKey.keypair;
  let [poolKey, poolBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintAKeypair.publicKey.toBuffer(),
      values.mintBKeypair.publicKey.toBuffer(),
      superUserKey.pubKey.toBuffer(),
      lpFeeBuffer,

    ],
    program.programId
  );


  let [lpTokenKey, lpTokenBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintAKeypair.publicKey.toBuffer(),
      values.mintBKeypair.publicKey.toBuffer(),
      superUserKey.pubKey.toBuffer(),
      Buffer.from("liquidity"),
    ],
    program.programId
  );


  let [mintAVaultKey, mintAVaultBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintAKeypair.publicKey.toBuffer(),
      poolKey.toBuffer(),
    ],
    program.programId
  );

  let [mintBVaultKey, mintBVaultBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintBKeypair.publicKey.toBuffer(),
      poolKey.toBuffer(),
    ],
    program.programId
  );

  let [mintATreasuryKey, mintATreasuryBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintAKeypair.publicKey.toBuffer(),
      poolKey.toBuffer(),
      Buffer.from("treasury"),
      superUserKey.pubKey.toBuffer(),
    ],
    program.programId
  );

  let [mintBTreasuryKey, mintBTreasuryBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      values.mintBKeypair.publicKey.toBuffer(),
      poolKey.toBuffer(),
      Buffer.from("treasury"),
      superUserKey.pubKey.toBuffer(),
    ],
    program.programId
  );

  //Setting the account structure
  let accounts={
    pool: poolKey,
    admin: superUserKey.pubKey,
    mintLiquidity: lpTokenKey,
    mintA: values.mintAKeypair.publicKey,
    mintB: values.mintBKeypair.publicKey,
    poolAccountA: mintAVaultKey,
    poolAccountB: mintBVaultKey,
    treasuryMintA: mintATreasuryKey,
    treasuryMintB: mintBTreasuryKey,
    payer:superUserKey.pubKey,
    depositorAccountLiquidity: values.liquidityAccount,
    depositorAccountA: values.holderAccountA,
    depositorAccountB: values.holderAccountB,
    traderAccountA: values.holderAccountA,
    traderAccountB: values.holderAccountB,
    depositor: superUserKey.pubKey,
  }

  it("Pool Creation", async () => {

    //Minting 100 token A and token B to the superUser
    await mintingTokens({
      connection,
      creator: superUser,
      mintAKeypair: values.mintAKeypair,
      mintBKeypair: values.mintBKeypair,
    });
    
    // Creating the pool
    let pool_lp_fee=new BN(new_pool_lp_fee_in_bp)
    await program.methods
      .createPool(10,poolBump,mintAVaultBump,mintBVaultBump,mintATreasuryBump,mintBTreasuryBump)
      .accounts(accounts)
      .rpc();


  });




  it("Creating uas ", async () => {

    const lptokenUTA = await getOrCreateAssociatedTokenAccount(
      connection,
      superUser,
      lpTokenKey,
      superUser.publicKey,
      true
    );
    
    const minAUTA = await getOrCreateAssociatedTokenAccount(
      connection,
      superUser,
      values.mintAKeypair.publicKey,
      superUser.publicKey,
      true
    );

    const minBUTA = await getOrCreateAssociatedTokenAccount(
      connection,
      superUser,
      values.mintBKeypair.publicKey,
      superUser.publicKey,
      true
    );


    accounts.depositorAccountA= minAUTA.address
    accounts.depositorAccountB= minBUTA.address
    accounts.depositorAccountLiquidity= lptokenUTA.address
    

    accounts.traderAccountA= minAUTA.address
    accounts.traderAccountB= minBUTA.address

    

  });
  
  it("Deposit equal amounts first deposit", async () => {

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );

    const traderLPToken_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountLiquidity
    );

    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    console.log("Depositing ",values.depositAmountA.toString()," of token A ", values.depositAmountA.toString()," of token B")
    await program.methods
      .depositLiquidity(values.depositAmountA, values.depositAmountA,new BN(0))
      .accounts(accounts)
 
      .rpc();

      const traderTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountA
      );
      const traderTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountB
      );
  
      const traderLPToken_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountLiquidity
      );
  
      const poolTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.poolAccountA
      );
      const poolTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.poolAccountB
      );

      console.log("Token A before and after user side",traderTokenAccountA_before.value.uiAmount,traderTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after user side",traderTokenAccountB_before.value.uiAmount,traderTokenAccountB_after.value.uiAmount,)
      console.log("Token A before and after pool side",poolTokenAccountA_before.value.uiAmount,poolTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after pool side",poolTokenAccountB_before.value.uiAmount,poolTokenAccountB_after.value.uiAmount,)
    
      console.log("LP token before and after user side",traderLPToken_before.value.uiAmount,traderLPToken_after.value.uiAmount,)
  
    });



  it("Swap from A to B", async () => {
    const input = new BN(10 ** 6);

    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );



    console.log("TOKEN A start : ",poolTokenAccountA_before.value.uiAmount)
    console.log("TOKEN B start : ",poolTokenAccountB_before.value.uiAmount)
    
    let tx1= await program.methods
      .simpleSwapExactIn(false, input, new BN(100))
      .accounts(accounts)
      .rpc();

    const traderTokenAccountA = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );


    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("SWAP DETAILS",txDetails.meta.logMessages)

    if(true){

      let impact_token_A=Number(traderTokenAccountA.value.amount)-Number(traderTokenAccountA_before.value.amount)
      let impact_token_B=Number(traderTokenAccountB.value.amount)-Number(traderTokenAccountB_before.value.amount)
      console.log("TOKEN A impact : ",impact_token_A)
      console.log("TOKEN B impact : ",impact_token_B)
      
    }
  });


  
  it("Deposit equal amounts second deposit", async () => {

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );

    const traderLPToken_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountLiquidity
    );

    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    let invariant=Number(poolTokenAccountA_before.value.amount)*Number(poolTokenAccountB_before.value.amount)
    console.log("invariant",invariant)
    let amountB=invariant/Number(values.depositAmountA)
    console.log("amountB",amountB,values.depositAmountA)
    console.log("Depositing ",values.depositAmountA.toString()," of token A ", values.depositAmountA.toString()," of token B")
    let tx1=await program.methods
      .depositLiquidity(values.depositAmountA, values.depositAmountA,new BN(0))
      .accounts(accounts)
  
      .rpc();

      get_onchain_logs(connection,tx1)
      const traderTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountA
      );
      const traderTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountB
      );
  
      const traderLPToken_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountLiquidity
      );
  
      const poolTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.poolAccountA
      );
      const poolTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.poolAccountB
      );

      console.log("Token A before and after user side",traderTokenAccountA_before.value.uiAmount,traderTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after user side",traderTokenAccountB_before.value.uiAmount,traderTokenAccountB_after.value.uiAmount,)
      console.log("Token A before and after pool side",poolTokenAccountA_before.value.uiAmount,poolTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after pool side",poolTokenAccountB_before.value.uiAmount,poolTokenAccountB_after.value.uiAmount,)
      console.log("LP token before and after user side",traderLPToken_before.value.uiAmount,traderLPToken_after.value.uiAmount,)
  
    });

    

  it("Swap from B to A", async () => {
    const input = new BN(10 ** 6);

    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );



    console.log("TOKEN A start : ",poolTokenAccountA_before.value.uiAmount)
    console.log("TOKEN B start : ",poolTokenAccountB_before.value.uiAmount)
    
    let tx1= await program.methods
      .simpleSwapExactIn(true, input, new BN(100))
      .accounts(accounts)
      .rpc();

    const traderTokenAccountA = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );



    if(true){

      let impact_token_A=Number(traderTokenAccountA.value.amount)-Number(traderTokenAccountA_before.value.amount)
      let impact_token_B=Number(traderTokenAccountB.value.amount)-Number(traderTokenAccountB_before.value.amount)
      console.log("TOKEN A impact : ",impact_token_A)
      console.log("TOKEN B impact : ",impact_token_B)
      
    }
  });



  it("First Withdraw everything", async () => {


    
    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );



    console.log("TOKEN A start in pool : ",poolTokenAccountA_before.value.uiAmount)
    console.log("TOKEN B start in pool : ",poolTokenAccountB_before.value.uiAmount)
    console.log("TOKEN A start in user wallet : ",traderTokenAccountA_before.value.uiAmount)
    console.log("TOKEN B start in user wallet : ",traderTokenAccountB_before.value.uiAmount)



    const traderLPToken_after = await connection.getTokenAccountBalance(
      accounts.depositorAccountLiquidity
    );

    let tx1=await program.methods
      .withdrawLiquidity(new BN(traderLPToken_after.value.amount),new BN(0),new BN(0))
      .accounts(accounts)
      .signers([superUser])
      .rpc();


    
      const poolTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.poolAccountA
      );
      const poolTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.poolAccountB
      );
  
      const traderTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountA
      );
      const traderTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountB
      );
  
  
  
      console.log("TOKEN A start in pool : ",poolTokenAccountA_after.value.uiAmount)
      console.log("TOKEN B start in pool : ",poolTokenAccountB_after.value.uiAmount)
      console.log("TOKEN A start in user wallet : ",traderTokenAccountA_after.value.uiAmount)
      console.log("TOKEN B start in user wallet : ",traderTokenAccountB_after.value.uiAmount)
  
  

  });

  it("Deposit equal amounts third deposit", async () => {

    const traderTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountA
    );
    const traderTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountB
    );

    const traderLPToken_before = await connection.getTokenAccountBalance(
      accounts.depositorAccountLiquidity
    );

    const poolTokenAccountA_before = await connection.getTokenAccountBalance(
      accounts.poolAccountA
    );
    const poolTokenAccountB_before = await connection.getTokenAccountBalance(
      accounts.poolAccountB
    );

    let invariant=Number(poolTokenAccountA_before.value.amount)*Number(poolTokenAccountB_before.value.amount)
    console.log("invariant",invariant)
    let amountB=invariant/Number(values.depositAmountA)
    console.log("amountB",amountB,values.depositAmountA)
    console.log("Depositing ",values.depositAmountA.toString()," of token A ", values.depositAmountA.toString()," of token B")
    let tx1=await program.methods
      .depositLiquidity(values.depositAmountA, values.depositAmountA,new BN(0))
      .accounts(accounts)
  
      .rpc();

      get_onchain_logs(connection,tx1)
      const traderTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountA
      );
      const traderTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountB
      );
  
      const traderLPToken_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountLiquidity
      );
  
      const poolTokenAccountA_after = await connection.getTokenAccountBalance(
        accounts.poolAccountA
      );
      const poolTokenAccountB_after = await connection.getTokenAccountBalance(
        accounts.poolAccountB
      );

      console.log("Token A before and after user side",traderTokenAccountA_before.value.uiAmount,traderTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after user side",traderTokenAccountB_before.value.uiAmount,traderTokenAccountB_after.value.uiAmount,)
      console.log("Token A before and after pool side",poolTokenAccountA_before.value.uiAmount,poolTokenAccountA_after.value.uiAmount,)
      console.log("Token B before and after pool side",poolTokenAccountB_before.value.uiAmount,poolTokenAccountB_after.value.uiAmount,)
      console.log("LP token before and after user side",traderLPToken_before.value.uiAmount,traderLPToken_after.value.uiAmount,)
  
    });

   
    it("Second withdraw everything", async () => {


    
      const poolTokenAccountA_before = await connection.getTokenAccountBalance(
        accounts.poolAccountA
      );
      const poolTokenAccountB_before = await connection.getTokenAccountBalance(
        accounts.poolAccountB
      );
  
      const traderTokenAccountA_before = await connection.getTokenAccountBalance(
        accounts.depositorAccountA
      );
      const traderTokenAccountB_before = await connection.getTokenAccountBalance(
        accounts.depositorAccountB
      );
  
  
  
      console.log("TOKEN A start in pool : ",poolTokenAccountA_before.value.uiAmount)
      console.log("TOKEN B start in pool : ",poolTokenAccountB_before.value.uiAmount)
      console.log("TOKEN A start in user wallet : ",traderTokenAccountA_before.value.uiAmount)
      console.log("TOKEN B start in user wallet : ",traderTokenAccountB_before.value.uiAmount)
  
  
  
      const traderLPToken_after = await connection.getTokenAccountBalance(
        accounts.depositorAccountLiquidity
      );
  
      let tx1=await program.methods
        .withdrawLiquidity(new BN(traderLPToken_after.value.amount),new BN(0),new BN(0))
        .accounts(accounts)
        .signers([superUser])
        .rpc();
  
  
        get_onchain_logs(connection,tx1)
        
        const poolTokenAccountA_after = await connection.getTokenAccountBalance(
          accounts.poolAccountA
        );
        const poolTokenAccountB_after = await connection.getTokenAccountBalance(
          accounts.poolAccountB
        );
    
        const traderTokenAccountA_after = await connection.getTokenAccountBalance(
          accounts.depositorAccountA
        );
        const traderTokenAccountB_after = await connection.getTokenAccountBalance(
          accounts.depositorAccountB
        );
    
    
    
        console.log("TOKEN A end in pool : ",poolTokenAccountA_after.value.uiAmount)
        console.log("TOKEN B end in pool : ",poolTokenAccountB_after.value.uiAmount)
        console.log("TOKEN A end in user wallet : ",traderTokenAccountA_after.value.uiAmount)
        console.log("TOKEN B end in user wallet : ",traderTokenAccountB_after.value.uiAmount)
    
    
  
    });
  
  


});
