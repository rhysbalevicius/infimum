# infimum-circuits

Fork of [maci-circuits](https://github.com/privacy-scaling-explorations/maci/tree/dev/packages/circuits).

```
mkdir -p build
```

## Compiling the circuits
```
circom ./main-process.circom --r1cs --wasm --sym -o build
circom ./main-tally.circom --r1cs --wasm --sym -o build
```

## Trusted setup
```
# Generate the Initial Powers of Tau
snarkjs powersoftau new bn128 19 ./build/pot12_0000.ptau -v

# Contribute to the Ceremony
snarkjs powersoftau contribute ./build/pot12_0000.ptau ./build/pot12_0001.ptau --name="First contribution" -v

# Prepare for Phase 2
snarkjs powersoftau prepare phase2 ./build/pot12_0001.ptau ./build/pot12_final.ptau -v

# Generate the Zero-Knowledge Proof Keys
snarkjs groth16 setup ./build/main-process.r1cs ./build/pot12_final.ptau ./build/process.zkey
snarkjs groth16 setup ./build/main-tally.r1cs ./build/pot12_final.ptau ./build/tally.zkey

# Export the Verification Key
snarkjs zkey export verificationkey ./build/process.zkey ./build/vk-process.json
snarkjs zkey export verificationkey ./build/tally.zkey ./build/vk-tally.json
```

## Prepare the input.json
For example:
```
{
    "preimage": "123456789",
    "hash": "7110303097080024260800444665787206606103183587082596139871399733998958991511"
}
```

## Generate the witness.wtns
```
node ./build/main-process_js/generate_witness.js ./build/main-process_js/main-process.wasm ./build/input-process.json ./build/witness-process.wtns
node ./build/main-tally_js/generate_witness.js ./build/main-tally_js/main-tally.wasm ./build/input-tally.json ./build/witness-tally.wtns
```

## Generate the Groth16 proof 
```
snarkjs groth16 prove ./build/process.zkey ./build/witness-process.wtns ./build/proof-process.json ./build/public-process.json
snarkjs groth16 prove ./build/tally.zkey ./build/witness-tally.wtns ./build/proof-tally.json ./build/public-tally.json
```

## Verify the proof
```
snarkjs groth16 verify ./build/vk-process.json ./build/public-process.json ./build/proof-process.json
snarkjs groth16 verify ./build/vk-tally.json ./build/public-tally.json ./build/proof-tally.json
```
