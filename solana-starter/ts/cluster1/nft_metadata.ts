import wallet from "../wba-wallet.json";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createGenericFile,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";

// Create a devnet connection
const umi = createUmi("https://api.devnet.solana.com");

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader({ address: "https://devnet.irys.xyz" }));
umi.use(signerIdentity(signer));

(async () => {
  try {
    // Follow this JSON structure
    // https://docs.metaplex.com/programs/token-metadata/changelog/v1.0#json-structure

    const image =
      "https://devnet.irys.xyz/Gw4Q4icvtrUKdmdA7qwujhpuiVZZuJJSDeEjqRP8H5eE";
    const metadata = {
      name: "Awesome Rug #1",
      symbol: "AR",
      description:
        "Get these Awesome Rug on https://github.com/deanmlittle/generug!",
      image,
      attributes: [{ trait_type: "Type", value: "Awesome" }],
      properties: {
        files: [
          {
            type: "image/jpg",
            uri: image,
          },
        ],
      },
      creators: [],
    };
    // const genericFile = createGenericFile(
    //   new TextEncoder().encode(JSON.stringify(metadata)),
    //   "metadata.json"
    // );
    // const myUri = await umi.uploader.upload([genericFile]);
    const myUri = await umi.uploader.uploadJson(metadata);
    console.log("Your metadata URI: ", myUri);
  } catch (error) {
    console.log("Oops.. Something went wrong", error);
  }
})();
