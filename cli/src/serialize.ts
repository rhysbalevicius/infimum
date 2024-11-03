import inf from 'inf-lib';
import path from 'path';
import { readFileSync } from 'fs';
import { Keypair, Message } from 'maci-domainobjs';
import { ProofData, PublicKey, VerifyingKey, VerifyingKeys } from './interface';

export const readJSON = (relPath: string) =>
{
    const file = readFileSync(path.join(__dirname, relPath)).toString();
    const data = JSON.parse(file);

    return data;
};

export const serializeProof = (relPath: string): ProofData =>
{
    const proof = readJSON(relPath);
    return inf.serialize_proof(proof);
};

const readVerifyingKey = (path: string) =>
{
    const {
        vk_alpha_1,
        vk_beta_2,
        vk_gamma_2,
        vk_delta_2,
        IC
    } = readJSON(path);

    return {
        vk_alpha_1,
        vk_beta_2,
        vk_gamma_2,
        vk_delta_2,
        ic: IC
    };
};

const serializeVerifyingKey = (path: string): VerifyingKey =>
{
    const data = readVerifyingKey(path);
    const vkey = inf.serialize_vkey(data);

    return vkey;
};

export const loadVerifyingKeys = (path: string): VerifyingKeys =>
{
    return {
        process: serializeVerifyingKey(`${path}/vk-process.json`),
        tally: serializeVerifyingKey(`${path}/vk-tally.json`)
    };
};

const padArrayStart = (array: Array<number>, fillValue: number, targetLength: number) =>
{
    return [ ...(Array(Math.max(0, targetLength - array.length)).fill(fillValue)), ...array ]
};

export const bnToBytes = (value: BigInt | string): Array<number> => padArrayStart(inf.bn_to_bytes_be({ value: value.toString() }).value, 0, 32);

export const serializePublicKey = (keypair: Keypair): PublicKey =>
{
    const [x, y] = keypair.pubKey.rawPubKey;
    const xBytes = inf.bn_to_bytes_be({ value: x.toString() }).value;
    const yBytes = inf.bn_to_bytes_be({ value: y.toString() }).value;
    return {
        x: padArrayStart(xBytes, 0, 32),
        y: padArrayStart(yBytes, 0, 32)
    };
};

export const serializeMessage = (message: Message): Array<Array<number>> =>
{
    return message.data.map(value => padArrayStart(inf.bn_to_bytes_be({ value: value.toString() }).value, 0, 32));
};
