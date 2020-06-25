# Distaff assembly
Distaff assembly is a simple, low-level language for writing programs for Distaff VM. It stands just above raw Distaff VM [instructions](isa.md), and in fact, many instructions in Distaff assembly map directly to instruction of Distaff VM. However, Distaff assembly has several advantages:

* Distaff assembly supports *macro instructions*. These instructions expand into sequences of raw Distaff VM instructions making it easier to encode common tasks.
* Distaff assembler takes care of properly aligning and padding all instructions reducing the amount of mental bookkeeping needed for writing programs.
* Distaff assembly natively supports flow control expression which the assembler automatically transforms into program execution graphs needed by Distaff VM.

### Compiling assembly code
To compile Distaff assembly source code into a program for Distaff VM, you can use the `compile()` function from the [assembly](https://github.com/GuildOfWeavers/distaff/blob/master/src/programs/assembly/mod.rs) module. This function takes the following parameters:

* `source: &str` - a reference to a string containing Distaff assembly source code.
* `hash_fn: HashFunction` - a hash function to be used for building a Merkle tree of program execution paths.

The `compile()` function returns `Result<Program, AssemblyError>` which will contain the compiled program if the compilation was successful, or if the source code contained errors, description of the first encountered error.

For example:
```Rust
use distaff::{ assembly, crypto::hash::blake3 };

// the program pushes values 3 and 5 onto the stack and adds them
let program = assembly::compile("push.3 push.5 add", blake3).unwrap();
```

## Assembly programs
A Distaff assembly program is just a sequence of instructions each describing a specific operation. You can use any combination of whitespace characters to separate one instruction from another.

All currently available instructions are described below. Many instructions can be parametrized with a single parameter. The notation for specifying parameters is *operation.parameter*. For example, `push.123` describes a `push` operation which is parametrized with value `123`.

For most instructions which support parameters, the default parameter is set to `1`. For example, `dup` is equivalent to `dup.1`, `choose` is equivalent to `choose.1` and so on.

A single instruction may take multiple VM cycles to execute. The number of cycles frequently depends on the specified parameter, and sometimes depends on other factors (e.g. place of the operation in the execution path). The tables below include this number of cycles in the last column.

### Flow control instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| noop      | Does nothing.                          | 1      |
| assert    | Pops the top item from the stack and checks if it is equal to `1`. If it is not equal to `1`, the program fails. | 1 |
| if.true   | Marks the beginning of the *true* branch in the `if.true else endif` expression. If the value at the top of the stack is `1`, the *true* branch is executed. | 1 |
| else      | Marks the beginning of the *false* branch in the `if.true else endif` expression. If the value at the top of the stack is `0`, the *false* branch is executed. | 2 |
| endif     | Marks the end of the `if.true else endif` expression.  | 0 |

#### If.true else endif expression
One of the ways to describe conditional execution in Distaff assembly is the `if.true else endif` expression. It can be used like so:
```
if.true
    <instructions>
else
    <instructions>
endif
```
where `instructions` can be a sequence of any instructions, including nested `if.true else endif` expressions. The above does the following:

1. Pops the top item from the stack.
2. If the value of the item is `1`, instructions following `if.true` instruction are executed.
3. If the value of the item is `0`, instructions following `else` instruction are executed.

A few important notes about the semantics of this operation:
* If the value at the top of the stack is not binary (i.e. not `0` or `1`), the operation will fail.
* Both branches of the expression must be specified, and there must be at least one instruction in each branch (even if just a `noop`).
* The assembler creates a separate execution path for each of the branches.

The last point means that instructions in the not taken branch are never executed. This is very efficient to execute, but in the current implementation, this results in doubling of possible execution paths for every `if.true else endif` expression. For example, having 10 `if.true else endif` expressions in your program will result in about 1000 possible execution paths.

The more possible execution paths there are, the longer a program will take to compile, and the more resources it will occupy in memory. Going over 20K - 30K possible execution paths, is probably not going to be practical. One way to get around this is by using [selection instructions](#Selection-instructions) instead of `if.true else endif` expressions, but this has performance implications of its own. In the future, a more performant implementation of `if.true else endif` expression will be provided.

### Input instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| push.*x*  | Pushes *x* onto the stack. *x* can be any valid field element. | 2 |
| read.a    | Pushes the next value from the input tape `A` onto the stack. | 1 |
| read.ab   | Pushes the next values from input tapes `A` and `B` onto the stack. Value from input tape `A` is pushed first, followed by the value from input tape `B`. | 1 |

#### Input tapes
Distaff VM has two input tapes for supplying secret inputs to a program: tape `A` and tape `B`. You can use `read.a` and `read.ab` instructions to move value from these tapes onto the stack. When a value is read from a tape, tape pointer advances to the next value. This means, that a value can be read from a tape only once. If you try to read values from a tape which has no more values, the operation will fail.

### Stack manipulation instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| dup.*n*   | Pushes copies of the top *n* stack items onto the stack. *n* can be any integer between 1 and 4. | 1 - 3 |
| pad.*n*   | Pushes *n* `0`'s onto the stack; *n* can be any integer between 1 and 8. | 1 - 4 |
| pick.*n*  | Pushes a copy of the item with index *n* onto the stack. For example, assuming `S0` is the top of the stack, executing `pick.2` transforms `S0 S1 S2 S3` into `S2 S0 S1 S2 S3`. *n* can be any integer between 1 and 3. | 2 - 5 |
| drop.*n*  | Removes top *n* items from the stack; *n* can be any integer between 1 and 8. | 1 - 3 |
| swap.1    | Moves the second from the top stack item to the top of the stack (swaps top two stack items). | 1 |
| swap.2    | Moves 3rd and 4th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S2 S3 S0 S1`. | 1 |
| swap.4    | Moves 5th through 8th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S4 S5 S6 S7 S0 S1 S2 S3`. | 1 |
| roll.4    | Moves 4th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S3 S0 S1 S2`. | 1 |
| roll.8    | Moves 8th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S7 S0 S1 S2 S3 S4 S5 S6`. | 1 |

### Arithmetic and boolean instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| add       | Pops top two items from the stack, adds them, and pushes the result onto the stack. | 1 |
| sub       | Pops top two items from the stack, subtracts the 1st item from the 2nd item, and pushes the result onto the stack.  | 2 |
| mul       | Pops top two items from the stack, multiplies them, and pushes the result onto the stack. | 1 |
| div       | Pops top two items from the stack, divides the 2nd item by the 1st item, and pushes the result onto the stack. If the item at the top of the stack is `0`, this operation will fail. | 2 |
| neg       | Pops the top item from the stack, computes its additive inverse, and pushes the result onto the stack. | 1      |
| inv       | Pops the top item from the stack, computes its multiplicative inverse, and pushes the result onto the stack. If the value at the top of the stack is `0`, this operation will fail. | 1 |
| not       | Pops the top item from the stack, subtracts it from value `1` and pushes the result onto the stack. In other words, `0` becomes `1`, and `1` becomes `0`. If the item at the top of the stack is not binary (i.e. not `0` or `1`), this operation will fail. | 1 |

#### Finite field arithmetic
All arithmetic operations in Distaff VM happen in a [prime field](https://en.wikipedia.org/wiki/Finite_field) with modulus `340282366920938463463374557953744961537` (which can also be written as 2<sup>128</sup> - 45 * 2<sup>40</sup> + 1). This means that overflow happens after a value exceeds field modulus. So, for example: `340282366920938463463374557953744961536 + 1 = 0`.

Divisions in prime fields are defined as inverse of multiplication. Specifically, `c = a / b` means: find such `c` that `b * c = a`. This may lead to unintuitive results. For example, `1 / 2 = 170141183460469231731687278976872480769`.

### Comparison instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| eq        | Pops top two items from the stack, compares them, and if their values are equal, pushes `1` onto the stack; otherwise pushes `0` onto the stack. | 1 |
| gt.*n*    | Pops top two items from the stack, compares them, and if the 1st value is greater than the 2nd value, pushes `1` onto the stack; otherwise pushes `0` onto the stack. If either of the values is greater than 2<sup>*n*</sup>, the operation will fail. *n* can be any integer between 4 and 128. | *n + 15* |
| lt.*n*    | Pops top two items from the stack, compares them, and if the 1st value is less than the 2nd value, pushes `1` onto the stack; otherwise pushes `0` onto the stack. If either of the values is greater than 2<sup>*n*</sup>, the operation will fail. *n* can be any integer between 4 and 128. | *n + 14* |
| rc.*n*    | Pops the top item from the stack, checks if it is less than 2<sup>*n*</sup>, and if it is, pushes `1` onto the stack; otherwise pushes `0` onto the stack. *n* can be any integer between 4 and 128.| *n + 6* |

### Selection instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| choose.1  | Pops top 3 items from the stack, and pushes either the 1st or the 2nd value back onto the stack depending on whether the 3rd value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 1` becomes `S0`, while `S0 S1 0` becomes `S1`. This operation will fail if the 3rd stack item is not a binary value. | 1 |
| choose.2  | Pops top 6 items from the stack, and pushes either the 1st or the 2nd pair of values back onto the stack depending on whether the 5th value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 1 S5` becomes `S0 S1`, while `S0 S1 S2 S3 0 S5` becomes `S2 S3` (notice that `S5` is discarded in both cases). This operation will fail if the 5th stack item is not a binary value. | 1 |

#### Conditional execution
Selection instructions can be used to simulate conditional execution. This, in turn, can be used to eliminate simple `if.true else endif` expressions. For example, if we have a program with conditional branches which looks like so:
```
if.true
    <instructions>
else
    <instructions>
endif
```
We can transform it into a linear program using selection instructions like so:

1. First, execute instructions in the `if.true` branch and leave the result on the stack.
2. Then, execute instructions in the `else` branch and leave the result on the stack.
3. Finally, use `choose` or `choose.2` instruction to select between the two results based on the desired condition.

For simple programs, such transformation would not be difficult, but for complex programs with deeply nested conditional logic, this will be far from trivial. Moreover, this approach suffers from the fact that both branches of the program need to be executed, and this reduces efficiency. Nevertheless, for simple conditional expressions this approach may be preferred over `if.true else endif` expressions because it does not increase the number of possible program execution paths.

### Cryptographic instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| hash.*n*  | Pops top *n* items from the stack, computes their hash using [Rescue hash function](#Rescue-hash-function), and pushes the result onto the stack. The result is always represented by 2 stack items. *n* can be any integer between 1 and 4. | ~ 16 |
| mpath.*n* | Pops top 2 items from the stack, uses them to compute a root of a Merkle authentication path for a tree of depth *n*, and pushes the result onto the stack. The result is always represented by 2 stack items. Input tapes `A` and `B` are expected to contain nodes of the Merkle authentication path (see [here](#Merkle-authentication-path) for more info).  | ~ *32n* |

#### Rescue hash function
Distaff VM uses a modified version of [Rescue](https://eprint.iacr.org/2019/426) hash function. This modification adds half-rounds to the beginning and to the end of the standard Rescue hash function to make the arithmetization of the function fully foldable. High-level pseudo-code for the modified version looks like so:
```
for 10 iterations do:
    add round constants;
    apply s-box;
    apply MDS;
    add round constants;
    apply inverse s-box;
    apply MDS;
```
This modification should not impact security properties of the function, but it is worth noting that it has not been studied to the same extent as the standard Rescue hash function.

Parameters used for the hash function are:
* State width of 6 elements: 4 elements for rate + 2 elements for capacity.
* S-Box of power 3, though, in the future this may be changed to S-Box of power 5.


#### Merkle authentication path
As mentioned above, `mpath` instruction can be used to compute roots of Merkle authentication paths, but the semantics of this instruction are somewhat complicated and deserve a bit more explanation.

First, suppose we have a Merkle tree of depth 3 which looks like so:
```
           abcd
          /    \
        ab      cd
       /  \    /  \
      a    b  c    d
```
where: `ab = hash(a, b)`, `cd = hash(c, d)`, and `abcd = hash(ab, cd)`. All of these values are 256 bits in size, and thus, we'd need two 128-bit field elements to represent each of them in Distaff VM.

If we consider leaf `c`, Merkle authentication path for this leaf would be: [`d`, `ab`], and the root of this path would be `abcd`. To compute this root in Distaff VM we can use `mpath` instruction like so:

1. First, we need to put two elements representing leaf `c` onto the stack.
2. Then, we need to execute `mpath.3` instruction. We set the parameter to `3` because the depth of our Merkle tree is 3.
3. The result of the operation will be the value of `abcd` sitting in the top two registers of the stack.

For the above to work, we also need to populate input tapes `A` and `B` with additional data. Specifically, these tapes should contain:

1. Values of Merkle path nodes `d` and `ab`. Since these values are 256 bits each, we need to split each value across tapes `A` and `B`. For example, `d` will be represented by two 128-bit values: d<sub>0</sub> and d<sub>1</sub>.
2. Binary decomposition of `c`'s index in the tree. In our example, this index is 2, and its binary representation is `10`. Starting with the least significant bit, each bit should be put into a separate slot on tape `A`, interlaced with nodes of the Merkle path.

Applying the above to our example, we'd get inputs tapes looking like so:

| A               | B              |
| --------------- | -------------- |
| d<sub>0</sub>   | d<sub>1</sub>  |
| 0               | 0              |
| ab<sub>0</sub>  | ab<sub>1</sub> |
| 1               | 0              |

Here is a brief explanation:
* First, we put the value `d` represented by d<sub>0</sub> and d<sub>1</sub> into tapes `A` and `B`.
* Then we put the least significant bit of `c`'s index (which is `0`) into tape `A`, and also complement it with `0` in tape `B`.
* Next, we put the value `ab` represented by ab<sub>0</sub> and ab<sub>1</sub> into tapes `A` and `B`.
* Finally, we put the next bit of `c`'s index (which is `1`) into tape `A`, and complement it with `0` in tape `B`.

Note that even though we use only tape `A` for bits of `c`'s index, we always complement these inputs with `0`'s in tape `B`.

To summarize: if our input tapes are set up as shown above, and if our stack state is [c<sub>1</sub>, c<sub>0</sub>], where c<sub>1</sub> is at the top of the stack, executing `mpath.3` will transform the stack into [abcd<sub>1</sub>, abcd<sub>0</sub>].