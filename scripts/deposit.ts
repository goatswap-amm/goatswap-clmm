import * as anchor from "@coral-xyz/anchor";
import { NATIVE_MINT } from "@solana/spl-token";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  Percentage,
  buildGoatswapClient,
  getProgramConfigs,
} from "../sdk/src";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  console.log(provider.wallet.publicKey);

  const ctx = GoatswapContext.withProvider(
    provider,
    AddressUtil.toPubKey(getProgramConfigs[Network.Devnet].programId),
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const client = buildGoatswapClient(ctx);

  const poolAddress = "7tTFQKQ5n9YXWu5B3VYiBk83kEcH7W7Cot5xVLVizHas";

  const pool = await client.getPoolByTokens(
    "So11111111111111111111111111111111111111112",
    "8xGk1sVP3PikCPGdViGTnVLEukHGrQta5gSYpd1dw58j"
  );

  console.log(pool.getData());

  console.log(
    pool.reserve0().toString(),
    pool.reserve1().toString(),
    pool.getData().lpSupply.toString()
  );

  const quote = await pool.getDepositQuote(
    new anchor.BN(100000000),
    NATIVE_MINT,
    new Percentage(new anchor.BN(0), new anchor.BN(100))
  );

  console.log(
    `Quote: Input amount=${quote.inputAmount.toString()}, Other amount=${quote.otherAmount.toString()}, Lp amount=${quote.lpAmount.toString()}`
  );

  const { tx } = await pool.deposit(ctx.wallet.publicKey, quote);

  console.log("Sending transaction...");

  // // const id = await tx.buildAndExecute(undefined, {
  // //   skipPreflight: true,
  // //   maxRetries: 0,
  // // });

  // console.log(`Deposit done: ${id}`);
}

main();
