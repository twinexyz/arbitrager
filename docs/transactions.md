# Transactions Filter

All the transactions in Twine L2 node can be classified into the following groups.
1. L1 Deposit Transactions
2. L1 Forced Withdrawal Transactions
3. Layer Zero Transactions
4. Other Transactions

### **L1 Deposit Transactions**
These transactions originate on the L1, and are processed on L2. Once processed, L1 needs to be sure that the transactions were processed. So, when the block is commited back on L1, the L1 contracts verifies that these transactions has actually been executed on L2. 

### **L1 Forced Withdrawal Transactions**
These transactions again originate on L1, and these are to withdraw the funds bridged from L1, back to L2. These, again needs to be verified on L1, so see if the particular transaction has been included on L2. The flow of these transactions are similar to that of L1, however, deposit transactions normally should be successful on L2, because the funds locked on L1 is minted on L2. However, L1 does not have idea on the fund amount on L2 and cannot be validated. So, if user wants to force-withdraw 10 tokens from L2, and they have only 5 tokens, this transaction would fail.

### **Layer Zero Transactions**
Twine acts as a DVN for layer zero for the chainsm Twine supports as L1. Twine verifies the consensus of all the supported L1s, and then verifies the respective transaction as well before executing, which ensures only the valid transactions are processed. Similarly, the transactions on Twine are settled on all the L1s, means, we do relay message from one L1 to another L1 reliably and securely. This allows us to act as a DVN for Layer Zero. We filter LayerZero transactions, which wants to use Twine as a DVN, and relay message to other chain via this process. These are handled separately, as these needs to be notified to the Layer Zero contract.

### **Other Transactions**
Other regular transactions do not need to be handled separately. These transactions are just committed to L1, transaction hash is calculated, and all the transactions in a block (including the transactions mentioned above) are verified against the transaction root of the block.

