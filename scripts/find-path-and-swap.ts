import * as anchor from "@coral-xyz/anchor";

import { PublicKey } from "@solana/web3.js";
import { AddressUtil, GoatswapContext, Percentage } from "../sdk/src";
import { Network, getProgramConfigs } from "../sdk/src/config";
import {
  GoatswapRouterBuilder,
  applySlippageForRoute,
} from "../sdk/src/router/public";
import { calculatePriceImpact } from "../sdk/src/router/public";
import { PoolGraphBuilder } from "../sdk/src/utils/graphs/public";

const devnetConfig = {
  program: new PublicKey("HKwqLZQw1fcnnFds4nkxYAmYK67TvtZ6TnVLUMJviWPL"),
  alt: new PublicKey("2BsRCvR5i3NSM1YBHsGJrjbWjW1c5qgKEoNQLngUcYqA"),
};

const config = devnetConfig;

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const ctx = GoatswapContext.withProvider(
    provider,
    config.program,
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const pools = await ctx.program.account.poolState.all();
  const poolGraph = PoolGraphBuilder.buildPoolGraph(
    pools.map((pool) => ({
      address: pool.publicKey,
      token0Mint: pool.account.token0Mint,
      token1Mint: pool.account.token1Mint,
      token0Vault: pool.account.token0Vault,
      token1Vault: pool.account.token1Vault,
    }))
  );

  const startDate = new Date().getTime();

  const router = GoatswapRouterBuilder.buildWithPoolGraph(ctx, poolGraph);
  const routes = await router.findAllRoutes(Network.Devnet, {
    tokenIn: "4CmdFifW4JKUhYUmwtBKhrkczwSSHACEeY5tJ3gLbqkA",
    tokenOut: "7MUHZwRzh7s5Q1D6sMETZaaRNqYdJRkg5tShrwsEXq6y",
    tradeAmount: new anchor.BN(100),
    amountSpecifiedIsInput: true,
  });

  console.log(`Elapsed: ${new Date().getTime() - startDate}ms`);

  for (let i = 0; i < routes.length; i++) {
    const route = applySlippageForRoute(
      routes[i],
      new Percentage(new anchor.BN(10), new anchor.BN(100))
    );
    const priceImpact = calculatePriceImpact(
      route,
      getProgramConfigs[Network.Devnet].tradingFee,
      2
    );

    console.log(`Route #${i + 1}`);
    console.log(`├── Price impact: ${priceImpact}%`);
    console.log(
      `└── ${route.amountSpecifiedIsInput ? "Minimum received" : "Maximum sold"}: ${route.thresholdAmount.toString()}`
    );
    console.log(`-----------------------------------------------------------`);
    for (let j = 0; j < routes[i].subRoutes.length; j++) {
      const subRoute = routes[i].subRoutes[j];
      console.log(`Subroute #${j + 1}:`);
      console.log(
        `├── Token In  : ${AddressUtil.toPubKey(subRoute.inputMint.address).toString()}`
      );
      console.log(
        `├── Token Out : ${AddressUtil.toPubKey(subRoute.outputMint.address).toString()}`
      );
      console.log(
        `├── Pool      : ${AddressUtil.toPubKey(subRoute.poolState.address).toString()}`
      );
      console.log(`├── Amount In : ${subRoute.amountIn.toString()}`);
      console.log(`└── Amount Out: ${subRoute.amountOut.toString()}`);
    }
    console.log(
      `-----------------------------------------------------------\n`
    );
  }

  const bestRoute = routes[0];

  console.log("Performing swap...");
  const tx = await router.swap(
    applySlippageForRoute(
      bestRoute,
      new Percentage(new anchor.BN(10), new anchor.BN(100))
    ),
    null
  );

  // const lookupTable = (await ctx.connection.getAddressLookupTable(config.alt))
  //   .value;

  // const id = await tx.buildAndExecute(
  //   {
  //     lookupTableAccounts: [lookupTable],
  //   },
  //   {
  //     skipPreflight: true,
  //     maxRetries: 0,
  //   }
  // );

  // console.log(`Swap done: ${id}`);
}

main();
