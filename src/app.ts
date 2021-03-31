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
const DUMMY_ADDRESS = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
const SALT = "123";

// The addresses of deployed contracts are fixed, as long as the salt stays fixed
// const V1_ADDRESS = "5FEytjcx9nQYEiCJzFGZxrWjFNycV3CWbMQvDVMC1BdcL52S";
const PROXY_ADDRESS = "5CE1yuZx64tfiKgAeFhK6MePWfX3Q2nLG6ydbZSSVGFH85ud";

const endowment = 1230000000000000n;
const gasLimit = 100000n * 1000000n;

async function deployContract(
  api: any,
  keyPair: any,
  nonce: number,
  name: String,
  params: any
) {
  const wasm = fs.readFileSync(`target/ink/${name}/${name}.wasm`);
  const abi = JSON.parse(fs.readFileSync(`target/ink/${name}/metadata.json`));
  const code = new CodePromise(api, abi, wasm);

  await api.tx.contracts
    .instantiateWithCode(
      endowment,
      gasLimit,
      compactAddLength(code.code),
      params,
      "123"
    )
    .signAndSend(keyPair, { nonce: nonce });

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
  // const bobPair = keyring.addFromUri("//Bob");

  // Need to deploy a dummy contract first
  let nonce = 0;
  const [v1CodeHash, v1Abi] = await deployContract(
    api,
    alicePair,
    nonce,
    "v1",
    DUMMY_ADDRESS
  );
  console.log(`Deployed a V1 contract with hash ${v1CodeHash}`);

  nonce += 1;
  const [proxyCodeHash, proxyAbi] = await deployContract(
    api,
    alicePair,
    nonce,
    "proxy",
    v1CodeHash
  );

  console.log(`Deployed a proxy contract with hash ${proxyCodeHash}`);
  const proxyContract = new ContractPromise(api, proxyAbi, PROXY_ADDRESS);

  // insert some values
  nonce += 1;
  await proxyContract.tx
    .insert({ value: 0, gasLimit: gasLimit }, 3)
    .signAndSend(alicePair, { nonce: nonce });

  nonce += 1;
  await proxyContract.tx
    .insert({ value: 0, gasLimit: gasLimit }, 7)
    .signAndSend(alicePair, { nonce: nonce });

  nonce += 1;
  await proxyContract.tx
    .insert({ value: 0, gasLimit: gasLimit }, 8)
    .signAndSend(alicePair, { nonce: nonce });

  // Average should be the mean
  // const average = await proxyContract.query.average(ALICE, {
  //   value: 0,
  //   gasLimit: gasLimit,
  // });
  // assert(average == 6);

  nonce += 1;
  const [v2CodeHash, v2Abi] = await deployContract(
    api,
    alicePair,
    nonce,
    "v2",
    PROXY_ADDRESS
  );
  console.log(`Deployed a V2 contract with hash ${v2CodeHash}`);

  nonce += 1;
  await proxyContract.tx
    .upgrade({ value: 0, gasLimit: gasLimit }, v2CodeHash)
    .signAndSend(alicePair, { nonce: nonce });

  console.log(`Upgraded the inner contract to V2`);
  // Average should be a median now!
  // const average = await proxyContract.query.average(ALICE, {
  //   value: 0,
  //   gasLimit: gasLimit,
  // });
  // assert(average == 7);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
