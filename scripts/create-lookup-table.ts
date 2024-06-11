import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  AddressUtil,
  GoatswapContext,
  Network,
  PDAUtils,
  getProgramConfigs,
} from "../sdk/src";
import { createLookupTable } from "../utils";

const network = Network.Devnet;

const program = getProgramConfigs[network].programId;
const addresses = {
  programId: AddressUtil.toPubKey(program),
  eventAuthority: PDAUtils.getEventAuthority(program).publicKey,
  authority: PDAUtils.getAuthority(program).publicKey,
  ammConfig: AddressUtil.toPubKey(getProgramConfigs[network].ammConfig),
  feeReceiver: AddressUtil.toPubKey(getProgramConfigs[network].feeReceiver),
};

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const ctx = GoatswapContext.withProvider(
    provider,
    addresses.programId,
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const alt = await createLookupTable(
    ctx.connection,
    (provider.wallet as anchor.Wallet).payer,
    Object.values(addresses)
  );

  const lookupTable = (await provider.connection.getAddressLookupTable(alt))
    .value;

  console.log("Lookup table:", alt);
  console.log(lookupTable);
}

main();
