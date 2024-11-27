# Twine Aggregator

Twine aggregator aggregates proofs from the provers for twine node. Once it receives proofs from the prover servers, it verifies the proof and after that this service posts the transactions in block and the block execution proofs to all the L1s.

![Twine Aggregator Architecture](./assets/image.png)

> Note: Ensure docker is installed and is running. It's used to verify the proof. On the first proof verification, the required docker image should download automatically.

## Run Aggregator
1. Generate config based on config.yaml.
    > Note: You might have to give network access to mongodb atlas if you're using one.



2. Run the aggregator as
    ```sh
    cargo build --release
    cargo run --release -- --config temp-config.yaml run
    ```

    > Release flag is needed for sp1 verification.

3. Clients should send proof to server with the following json rpc request. 
    ```json
    {
    "jsonrpc": "2.0",
    "method": "twarb_sendProof",
    "params": [
        {
        "type": "SP1Proof",
        "identifier":"identifier1",
        "proof": "proof.json file contents of sp1"
        }
    ]
    }
    ```
    **Example**
    ```sh
    ./post_proof.sh assets/proof.json
    ```