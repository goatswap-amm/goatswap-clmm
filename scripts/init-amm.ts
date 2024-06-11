import * as anchor from "@coral-xyz/anchor";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  buildGoatswapClient,
  getProgramConfigs,
} from "../sdk/src";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const ctx = GoatswapContext.withProvider(
    provider,
    AddressUtil.toPubKey(getProgramConfigs[Network.Devnet].programId),
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const client = buildGoatswapClient(ctx);

  const owner = anchor.Wallet.local().payer;

  const configIndex = 1;
  const tradeFeeRate = 3000; //0.3%
  const protocolFeeRate = 250000; //25%
  const fundFeeRate = 250000;
  const createPoolFee = 1_000_000_000; // 1sol

  const { tx } = await client.createAMMConfig(owner.publicKey, {
    index: configIndex,
    tradeFeeRate,
    protocolFeeRate,
    fundFeeRate,
    createPoolFee,
  });

  console.log("Sending transaction...");

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Create AMM config done: tx=${id}`);
}

main();
