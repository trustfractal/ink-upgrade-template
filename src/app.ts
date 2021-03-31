import { ContractPromise } from "@polkadot/api-contract";
import { assert } from "node:console";

// Required imports
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { CodePromise, BlueprintPromise } = require("@polkadot/api-contract");
const fs = require("fs");
const { Keyring, decodeAddress, encodeAddress } = require("@polkadot/keyring");
import { compactAddLength } from "@polkadot/util";
import { blake2AsU8a, blake2AsHex } from "@polkadot/util-crypto";

// import proxy_abi from '../target/ink/proxy/metadata.json';
// import v1_abi from '../target/ink/v1/metadata.json';
// import v2_abi from '../target/ink/v2/metadata.json';
const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

// Deploy a contract using the Blueprint
const endowment = 1230000000000000n;

// NOTE The apps UI specifies these in Mgas
const gasLimit = 100000n * 1000000n;

async function deployContract(
  api: any,
  keyPair: any,
  name: String,
  params: any
) {
  const wasm = fs.readFileSync(`target/ink/${name}/${name}.wasm`);
  const abi = JSON.parse(fs.readFileSync(`target/ink/${name}/metadata.json`));
  const code = new CodePromise(api, abi, wasm);

  const contractAddress = await api.tx.contracts
    .instantiateWithCode(
      endowment,
      gasLimit,
      compactAddLength(code.code),
      params,
      "123"
    )
    .signAndSend(keyPair);

  console.log(`Raw result is ${contractAddress}`);
  console.log(`Decoded address is ${encodeAddress(contractAddress)}`);

  return [blake2AsHex(code.code), abi];
}

async function main() {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider("ws://127.0.0.1:9944");

  // Create the API and wait until ready
  const api = await ApiPromise.create({
    provider,
    types: { Address: "MultiAddress", LookupSource: "MultiAddress" },
  });

  const keyring = new Keyring({ type: "sr25519" });
  const alicePair = keyring.addFromUri("//Alice");

  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version(),
  ]);

  const [v1CodeHash, abi] = await deployContract(api, alicePair, "v1", ALICE);
  console.log(`V1 code hash is ${v1CodeHash}`);

  //   const contractPromise = new ContractPromise(api,
  // abi, decodeAddress(DELEGATOR_CONTRACT_ADDRESS));
  // }
  //   contracts.forEach(async (element) => {
  // deployContract(api, alicePair, element, "123");
  //   });

  console.log(
    `You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`
  );
}

main()
  .catch(console.error)
  .finally(() => process.exit());
