# Twine Aggregator

The **Twine Aggregator** is a critical component of the Twine L2 network, responsible for aggregating, verifying, and committing execution proofs from multiple block execution provers. These operations ensure that all blocks produced by the Twine node are properly verified and submitted to supported Layer 1 (L1) blockchains.

---

## **Overview**
Twine operates with a block time of under **15 seconds**, which demands high throughput and efficient proof generation. Since generating execution proofs for a single block takes approximately **3 minutes**, multiple **execution provers** are used in parallel to keep up with the pace of block production.

The aggregator coordinates the flow of execution proofs, verifies them, and commits the results to L1s.

---

![Aggregator](../assets/aggregator.png)

---


## **Workflow**

### **1. Block Proof Generation**
- The scheduler assigns block heights to execution provers.
- Provers work in parallel to generate SP1 proofs for their assigned blocks.
- Provers send SP1 proof of block execution to the aggregator.

### **2. Proof Verification**
- The aggregator verifies the SP1 proof sent by the prover node
- Once the proof is verified, the proof is saved to a DB for backup.
- For added security, we can have multiple provers geenrate proof for a same block. For this, we can have k / N threshold check, where once k proofs are verified, we go to the step below.

### **3. Block Commit**
- Once the proof is verified:
  - The aggregator extracts all transactions from the block and categorizes them.
  - It sends categorized transaction objects to the supported L1s for processing.

### **4. Groth16 Block Proof**
- After the block information is committed, the aggregator sends the corresponding Groth16 proof to the L1s to complete the block submission process.

---

## **Key Features**

### **1. High Throughput**
- Multiple execution provers operate concurrently to generate proofs, ensuring the system meets the demands of Twine’s block time.

### **2. Fault Tolerance**
- Aggregator retries proof submissions and L1 commits in case of transaction failures.

### **3. Transaction Categorization**
- Separates block transactions into distinct categories for better handling on L1s.
- Supports:
  - L1 Deposits
  - L1 Forced Withdrawals
  - Layer Zero DVN Transactions
  - Regular L2 Transactions
- This is explained in detail [here](./transactions.md)

### **4. Multi-L1 Support**
- The aggregator is compatible with multiple Layer 1 blockchains.
- Enables seamless interaction and synchronization across different L1s.
