pragma circom 2.0.0;

// NB this is probably for testing only. We will want to supply these parameters depending on the individual poll 
// Alternatively, since this is a PoC, maybe we can choose sensible fixed values...

include "./process-messages.circom";
component main = ProcessMessages(10, 2, 1, 2);

// include "./tally-votes.circom";
// component main = TallyVotes(10, 1, 2);
