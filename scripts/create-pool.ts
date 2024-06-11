import * as anchor from "@coral-xyz/anchor";
import {
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  buildGoatswapClient,
  getCurrentTime,
  getProgramConfigs,
} from "../sdk/src";

import { createTokenMint } from "../sdk/tests/utils/";
import { parseUnits } from "../sdk/src/common/math/bn-utils";

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

  const tokenA = await createTokenMint({
    connection: ctx.connection,
    mintAmount: 100_000_000,
    decimals: 9,
    user: (provider.wallet as anchor.Wallet).payer,
    transferFeeConfig: {
      transferFeeBasisPoints: 500,
      maxFee: 10000n * 10n ** 9n,
    },
    programId: TOKEN_2022_PROGRAM_ID,
  });

  const tokenB = NATIVE_MINT;

  const { tx, poolKey } = await client.initializePool(Network.Devnet, {
    creator: provider.wallet.publicKey,
    tokenAMint: tokenA,
    tokenBMint: tokenB,
    initAmountA: parseUnits(100_000, 9),
    initAmountB: parseUnits(1, 9),
    openTime: getCurrentTime(),
    taxMint: tokenB,
    inTaxRate: 50_000,
    outTaxRate: 100_000,
  });

  console.log("Sending transaction...");

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Create pool done: pool=${poolKey} tx=${id}`);
}

main();
