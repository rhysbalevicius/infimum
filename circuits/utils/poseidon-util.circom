pragma circom 2.1.5;

template Sigma()
{
    signal input in;
    signal output out;

    signal in2;
    signal in4;

    in2 <== in*in;
    in4 <== in2*in2;

    out <== in4*in;
}

template Ark(t, C, r)
{
    signal input in[t];
    signal output out[t];

    for (var i = 0; i < t; i++)
    {
        out[i] <== in[i] + C[i + r];
    }
}

template Mix(t, M)
{
    signal input in[t];
    signal output out[t];

    var lc;
    for (var i = 0; i < t; i++)
    {
        lc = 0;
        for (var j = 0; j < t; j++)
        {
            lc += M[i][j]*in[j];
        }
        out[i] <== lc;
    }
}
