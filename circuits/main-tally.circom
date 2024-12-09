pragma circom 2.0.0;

include "./tally-votes.circom";
component main {public [index, numSignUps, sbCommitment, currentTallyCommitment, newTallyCommitment]} = TallyVotes(10, 1, 2);
