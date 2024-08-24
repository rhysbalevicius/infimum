# circuits

```
mkdir -p build
```

## Compiling the circuit
```
circom ./circom/poseidon-preimage.circom --r1cs --wasm --sym -o build
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
snarkjs groth16 setup ./build/poseidon-preimage.r1cs ./build/pot12_final.ptau ./build/poseidon-preimage.zkey

# Export the Verification Key
snarkjs zkey export verificationkey ./build/poseidon-preimage.zkey ./build/verification-key.json
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
node ./build/poseidon-preimage_js/generate_witness.js ./build/poseidon-preimage_js/poseidon-preimage.wasm ./build/input.json ./build/witness.wtns
```

## Generate the Groth16 proof 
```
snarkjs groth16 prove ./build/poseidon-preimage.zkey ./build/witness.wtns ./build/proof.json ./build/public.json
```
