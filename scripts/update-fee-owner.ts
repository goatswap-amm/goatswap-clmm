import * as anchor from "@coral-xyz/anchor";

import {
  AMMConfigParam,
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
  const newProtocolFeeOwner = owner.publicKey;

  const { tx } = await client.updateAMMConfig(
    owner.publicKey,
    configIndex,
    AMMConfigParam.NewProtocolOwner,
    0,
    newProtocolFeeOwner
  );

  console.log("Sending transaction...");

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Update new fee owner done: tx=${id}`);
}

main();
