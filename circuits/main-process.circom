pragma circom 2.0.0;

include "./process-messages.circom";
component main {public [numSignUps, index, batchEndIndex, msgRoot, currentSbCommitment, newSbCommitment, pollEndTimestamp, actualStateTreeDepth, coordinatorPublicKeyHash]} = ProcessMessages(10, 2, 1, 2);
