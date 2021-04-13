// Required imports
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { ContractPromise, CodePromise } = require("@polkadot/api-contract");
const fs = require("fs");
const { Keyring } = require("@polkadot/keyring");
import { blake2AsU8a, blake2AsHex } from "@polkadot/util-crypto";

const endowment = 4000n * 1000000n * 1000000n;
const gasLimit = 200000n * 1000000n;

async function deployContract(
  api: any,
  keyPair: any,
  name: String,
  params: any[],
): Promise<any> {
  const wasm = fs.readFileSync(`target/ink/${name}/${name}.wasm`);
  const abi = JSON.parse(fs.readFileSync(`target/ink/${name}/metadata.json`));
  const code = new CodePromise(api, abi, wasm);

  const tx = code.tx.new({ gasLimit, value: endowment }, ...params);

  const contract = await waitForTx(api, keyPair, tx);

  return { abi, contract };
}

async function waitForTx(api: any, keyPair: any, tx: any) {
  return new Promise(async (resolve, reject) => {
    const { ExtrinsicFailed, ExtrinsicSuccess } = api.events.system;
    const { Instantiated } = api.events.contracts;

    const unsub = await tx.signAndSend(keyPair, (e: any) => {
      for (const { event } of Object.values(e.events) as any) {
        if (Instantiated.is(event)) {
          let { data: [deployer, address] } = event;
          resolve(e.contract);
        }

        if (ExtrinsicFailed.is(event)) {
          const error = event.data[0];
          if (error.isModule) {
            // for module errors, we have the section indexed, lookup
            const decoded = api.registry.findMetaError(error.asModule);
            const { documentation, method, section } = decoded;
            reject(`${section}.${method}: ${documentation.join(" ")}`);
          } else {
            reject(error.toString());
          }
        }

        if (ExtrinsicSuccess.is(event)) {
          resolve(event);
        }
      }

      if (e.status.isInBlock || e.status.isFinalized) resolve(null);
    });
  });
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
  const alice = keyring.addFromUri("//Alice");
  const bob = keyring.addFromUri("//Bob");

  // Need to deploy a dummy contract first
  const v1 = await deployContract(api, alice, "v1", [alice.address]);
  console.log("Deployed V1 contract:");
  console.log(`  hash: ${v1.abi.source.hash}`);
  console.log(`  address ${v1.contract.address}`);

  const v2 = await deployContract(api, alice, "v2", [alice.address]);
  console.log("Deployed V2 contract:");
  console.log(`  hash: ${v2.abi.source.hash}`);
  console.log(`  address ${v2.contract.address}`);

  const proxy = await deployContract(api, alice, "proxy", [v1.abi.source.hash]);
  console.log("Deployed proxy contract:");
  console.log(`  hash: ${proxy.abi.source.hash}`);
  console.log(`  address ${proxy.contract.address}`);

  // insert some values
  await waitForTx(api, alice, proxy.contract.tx.insert({ value: 0, gasLimit }, 3));
  await waitForTx(api, alice, proxy.contract.tx.insert({ value: 0, gasLimit }, 7));
  await waitForTx(api, alice, proxy.contract.tx.insert({ value: 0, gasLimit }, 8));

  // Average should be the mean
  const avgV1 = await proxy.contract.query.average(alice.address, { gasLimit });
  console.log("avg-v1:", avgV1.output.toString());
  console.assert(avgV1.output.toString() === "6");

  await waitForTx(api, alice, proxy.contract.tx.upgrade({ value: 0, gasLimit }, v2.abi.source.hash));
  console.log(`Upgraded the inner contract to V2`);

  // Average should be a median now!
  const avgV2 = await proxy.contract.query.average(alice.address, { gasLimit });
  console.log("avg-v2:", avgV2.output.toString());
  console.assert(avgV2.output.toString() === "7");
}

main().catch(console.error).finally(() => process.exit());
