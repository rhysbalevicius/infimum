import { Keypair, PrivKey } from 'maci-domainobjs';
import { ApiPromise, Keyring } from '@polkadot/api';
import { NetworkKeyring } from '../interface';
import { extrinsic } from '../extrinsic';

export class User
{
    /** Network connection. */
    protected api: ApiPromise;

    /** Keypair for private communication between participants and coordinator. */
    protected palletKeypair: Keypair;

    /** Keyring for interacting with the network. */
    protected networkKeyring: NetworkKeyring;

    protected sendExtrinsic: ReturnType<typeof extrinsic>;

    constructor(
        api: ApiPromise,
        keyringURI: string,
        privateKey: string
    )
    {
        const keyring = new Keyring({ type: 'sr25519' });

        this.api = api;
        this.networkKeyring = keyring.addFromUri(keyringURI);
        this.palletKeypair = new Keypair(new PrivKey(privateKey));
        this.sendExtrinsic = extrinsic(api, this.networkKeyring);
    }
    
    keypair() { return this.palletKeypair; };
    address() { return this.networkKeyring.address; }
}
