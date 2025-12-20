"""
fPOW - Algorand Proof of Work Mining Game
A PyTeal smart contract implementing the fPOW mining game on Algorand.
"""

from pyteal import *
from typing import Final

# Constants
TOKEN_DECIMALS: Final[int] = 11
ONE_FPOW: Final[int] = 10 ** TOKEN_DECIMALS
MAX_SUPPLY: Final[int] = ONE_FPOW * 5_000_000
DENOMINATOR_BPS: Final[int] = 10_000
CHECKPOINT_FEE: Final[int] = 10_000  # 0.01 ALGO in microalgos
ADMIN_FEE: Final[int] = 100  # 1% in basis points

# Box names
BOARD_BOX: Final[bytes] = b"board"
CONFIG_BOX: Final[bytes] = b"config"
TREASURY_BOX: Final[bytes] = b"treasury"
MINER_PREFIX: Final[bytes] = b"miner"
STAKE_PREFIX: Final[bytes] = b"stake"
ROUND_PREFIX: Final[bytes] = b"round"
AUTOMATION_PREFIX: Final[bytes] = b"automation"


def approval_program():
    """Main approval program for the fPOW application."""

    # Global state keys
    admin_key = Bytes("admin")
    fpow_asa_id_key = Bytes("fpow_asa_id")
    round_id_key = Bytes("round_id")

    # Scratch space
    scratch_amount = ScratchVar(TealType.uint64)
    scratch_sender = ScratchVar(TealType.bytes)

    # Helper to get miner box name
    @Subroutine(TealType.bytes)
    def miner_box_name(authority: Expr) -> Expr:
        return Concat(Bytes(MINER_PREFIX), authority)

    # Helper to get stake box name
    @Subroutine(TealType.bytes)
    def stake_box_name(authority: Expr) -> Expr:
        return Concat(Bytes(STAKE_PREFIX), authority)

    # Helper to get round box name
    @Subroutine(TealType.bytes)
    def round_box_name(round_id: Expr) -> Expr:
        return Concat(Bytes(ROUND_PREFIX), Itob(round_id))

    # Helper to get automation box name
    @Subroutine(TealType.bytes)
    def automation_box_name(authority: Expr) -> Expr:
        return Concat(Bytes(AUTOMATION_PREFIX), authority)

    # Initialize the application
    @Subroutine(TealType.none)
    def initialize():
        return Seq([
            App.globalPut(admin_key, Txn.sender()),
            App.globalPut(round_id_key, Int(0)),
            # Create board box
            Assert(App.box_create(Bytes(BOARD_BOX), Int(32))),
            # Create treasury box
            Assert(App.box_create(Bytes(TREASURY_BOX), Int(128))),
            # Create config box
            Assert(App.box_create(Bytes(CONFIG_BOX), Int(168))),
            Approve(),
        ])

    # Claim ALGO rewards
    @Subroutine(TealType.none)
    def claim_algo():
        miner_box = miner_box_name(Txn.sender())
        rewards_algo_offset = Int(32 + 200 + 200 + 8 + 8 + 8 + 8 + 16)  # Offset to rewards_algo field
        return Seq([
            # Load miner box
            Assert(App.box_length(miner_box) != Int(0)),
            # Get rewards amount
            scratch_amount.store(App.box_extract(miner_box, rewards_algo_offset, Int(8))),
            # Zero out rewards
            App.box_replace(miner_box, rewards_algo_offset, Itob(Int(0))),
            # Transfer ALGO to sender
            InnerTransactionBuilder.Begin(),
            InnerTransactionBuilder.SetFields({
                TxnField.type_enum: TxnType.Payment,
                TxnField.receiver: Txn.sender(),
                TxnField.amount: Btoi(scratch_amount.load()),
                TxnField.fee: Int(0),
            }),
            InnerTransactionBuilder.Submit(),
            Approve(),
        ])

    # Claim fPOW rewards
    @Subroutine(TealType.none)
    def claim_fpow():
        miner_box = miner_box_name(Txn.sender())
        fpow_asa = App.globalGet(fpow_asa_id_key)
        rewards_fpow_offset = Int(32 + 200 + 200 + 8 + 8 + 8 + 8 + 16 + 8)  # Offset to rewards_fpow field
        return Seq([
            # Load miner box
            Assert(App.box_length(miner_box) != Int(0)),
            # Get rewards amount
            scratch_amount.store(App.box_extract(miner_box, rewards_fpow_offset, Int(8))),
            # Zero out rewards
            App.box_replace(miner_box, rewards_fpow_offset, Itob(Int(0))),
            # Transfer fPOW ASA to sender
            InnerTransactionBuilder.Begin(),
            InnerTransactionBuilder.SetFields({
                TxnField.type_enum: TxnType.AssetTransfer,
                TxnField.asset_receiver: Txn.sender(),
                TxnField.asset_amount: Btoi(scratch_amount.load()),
                TxnField.xfer_asset: fpow_asa,
                TxnField.fee: Int(0),
            }),
            InnerTransactionBuilder.Submit(),
            Approve(),
        ])

    # Deploy to squares
    @Subroutine(TealType.none)
    def deploy():
        amount = Btoi(Txn.application_args[1])
        mask = Btoi(Txn.application_args[2])
        authority = Txn.accounts[1] if Txn.accounts.length() > Int(1) else Txn.sender()
        miner_box = miner_box_name(authority)
        board_box = Bytes(BOARD_BOX)
        return Seq([
            # Verify payment attached
            Assert(Gtxn[Txn.group_index() - Int(1)].type_enum() == TxnType.Payment),
            Assert(Gtxn[Txn.group_index() - Int(1)].receiver() == Global.current_application_address()),
            Assert(Gtxn[Txn.group_index() - Int(1)].amount() >= amount),
            # Create miner box if needed
            If(App.box_length(miner_box) == Int(0)).Then(
                Assert(App.box_create(miner_box, Int(512)))
            ),
            # Update miner deployed amounts based on mask
            # (simplified - full implementation would iterate through squares)
            Approve(),
        ])

    # Checkpoint miner rewards
    @Subroutine(TealType.none)
    def checkpoint():
        authority = Txn.accounts[1] if Txn.accounts.length() > Int(1) else Txn.sender()
        miner_box = miner_box_name(authority)
        return Seq([
            # Load miner box
            Assert(App.box_length(miner_box) != Int(0)),
            # Update checkpoint_id
            # (simplified implementation)
            Approve(),
        ])

    # Reset round
    @Subroutine(TealType.none)
    def reset():
        board_box = Bytes(BOARD_BOX)
        current_round = App.globalGet(round_id_key)
        return Seq([
            # Verify caller is authorized
            Assert(Txn.sender() == App.globalGet(admin_key)),
            # Increment round
            App.globalPut(round_id_key, current_round + Int(1)),
            # Create new round box
            Assert(App.box_create(round_box_name(current_round + Int(1)), Int(512))),
            Approve(),
        ])

    # Close round
    @Subroutine(TealType.none)
    def close():
        round_id = Btoi(Txn.application_args[1])
        round_box = round_box_name(round_id)
        return Seq([
            # Delete round box
            Assert(App.box_delete(round_box)),
            Approve(),
        ])

    # Deposit stake
    @Subroutine(TealType.none)
    def deposit():
        amount = Btoi(Txn.application_args[1])
        stake_box = stake_box_name(Txn.sender())
        return Seq([
            # Verify ASA transfer attached
            Assert(Gtxn[Txn.group_index() - Int(1)].type_enum() == TxnType.AssetTransfer),
            Assert(Gtxn[Txn.group_index() - Int(1)].asset_receiver() == Global.current_application_address()),
            Assert(Gtxn[Txn.group_index() - Int(1)].asset_amount() >= amount),
            # Create stake box if needed
            If(App.box_length(stake_box) == Int(0)).Then(
                Assert(App.box_create(stake_box, Int(256)))
            ),
            # Update stake balance
            # (simplified implementation)
            Approve(),
        ])

    # Withdraw stake
    @Subroutine(TealType.none)
    def withdraw():
        amount = Btoi(Txn.application_args[1])
        stake_box = stake_box_name(Txn.sender())
        fpow_asa = App.globalGet(fpow_asa_id_key)
        return Seq([
            # Load stake box
            Assert(App.box_length(stake_box) != Int(0)),
            # Transfer fPOW back to sender
            InnerTransactionBuilder.Begin(),
            InnerTransactionBuilder.SetFields({
                TxnField.type_enum: TxnType.AssetTransfer,
                TxnField.asset_receiver: Txn.sender(),
                TxnField.asset_amount: amount,
                TxnField.xfer_asset: fpow_asa,
                TxnField.fee: Int(0),
            }),
            InnerTransactionBuilder.Submit(),
            Approve(),
        ])

    # Claim yield
    @Subroutine(TealType.none)
    def claim_yield():
        amount = Btoi(Txn.application_args[1])
        stake_box = stake_box_name(Txn.sender())
        fpow_asa = App.globalGet(fpow_asa_id_key)
        return Seq([
            # Load stake box
            Assert(App.box_length(stake_box) != Int(0)),
            # Transfer yield to sender
            InnerTransactionBuilder.Begin(),
            InnerTransactionBuilder.SetFields({
                TxnField.type_enum: TxnType.AssetTransfer,
                TxnField.asset_receiver: Txn.sender(),
                TxnField.asset_amount: amount,
                TxnField.xfer_asset: fpow_asa,
                TxnField.fee: Int(0),
            }),
            InnerTransactionBuilder.Submit(),
            Approve(),
        ])

    # Compound yield
    @Subroutine(TealType.none)
    def compound_yield():
        stake_box = stake_box_name(Txn.sender())
        return Seq([
            # Load stake box
            Assert(App.box_length(stake_box) != Int(0)),
            # Add yield to stake balance
            # (simplified implementation)
            Approve(),
        ])

    # Set admin
    @Subroutine(TealType.none)
    def set_admin():
        new_admin = Txn.application_args[1]
        return Seq([
            Assert(Txn.sender() == App.globalGet(admin_key)),
            App.globalPut(admin_key, new_admin),
            Approve(),
        ])

    # Automate mining
    @Subroutine(TealType.none)
    def automate():
        amount = Btoi(Txn.application_args[1])
        deposit_amount = Btoi(Txn.application_args[2])
        fee = Btoi(Txn.application_args[3])
        mask = Btoi(Txn.application_args[4])
        strategy = Btoi(Txn.application_args[5])
        reload = Btoi(Txn.application_args[6])
        automation_box = automation_box_name(Txn.sender())
        return Seq([
            # Create automation box if needed
            If(App.box_length(automation_box) == Int(0)).Then(
                Assert(App.box_create(automation_box, Int(128)))
            ),
            # Store automation settings
            # (simplified implementation)
            Approve(),
        ])

    # Reload ALGO
    @Subroutine(TealType.none)
    def reload_algo():
        authority = Txn.accounts[1] if Txn.accounts.length() > Int(1) else Txn.sender()
        automation_box = automation_box_name(authority)
        miner_box = miner_box_name(authority)
        return Seq([
            # Load automation box
            Assert(App.box_length(automation_box) != Int(0)),
            # Transfer ALGO from miner to automation
            # (simplified implementation)
            Approve(),
        ])

    # Bury tokens
    @Subroutine(TealType.none)
    def bury():
        amount = Btoi(Txn.application_args[1])
        fpow_asa = App.globalGet(fpow_asa_id_key)
        return Seq([
            # Verify ASA transfer attached
            Assert(Gtxn[Txn.group_index() - Int(1)].type_enum() == TxnType.AssetTransfer),
            Assert(Gtxn[Txn.group_index() - Int(1)].asset_receiver() == Global.current_application_address()),
            Assert(Gtxn[Txn.group_index() - Int(1)].asset_amount() >= amount),
            # Burn tokens (send to zero address or delete)
            # (simplified implementation)
            Approve(),
        ])

    # Wrap ALGO
    @Subroutine(TealType.none)
    def wrap():
        amount = Btoi(Txn.application_args[1])
        return Seq([
            # Verify payment attached
            Assert(Gtxn[Txn.group_index() - Int(1)].type_enum() == TxnType.Payment),
            Assert(Gtxn[Txn.group_index() - Int(1)].receiver() == Global.current_application_address()),
            Assert(Gtxn[Txn.group_index() - Int(1)].amount() >= amount),
            # Store in treasury
            # (simplified implementation)
            Approve(),
        ])

    # Buyback
    @Subroutine(TealType.none)
    def buyback():
        return Seq([
            # Verify caller is authorized
            Assert(Txn.sender() == App.globalGet(admin_key)),
            # Execute buyback
            # (simplified implementation)
            Approve(),
        ])

    # Liquidity
    @Subroutine(TealType.none)
    def liq():
        return Seq([
            # Verify caller is authorized
            Assert(Txn.sender() == App.globalGet(admin_key)),
            # Execute liquidity operation
            # (simplified implementation)
            Approve(),
        ])

    # New var (entropy)
    @Subroutine(TealType.none)
    def new_var():
        var_id = Btoi(Txn.application_args[1])
        commit = Txn.application_args[2]
        samples = Btoi(Txn.application_args[3])
        return Seq([
            # Verify caller is authorized
            Assert(Txn.sender() == App.globalGet(admin_key)),
            # Store entropy var
            # (simplified implementation)
            Approve(),
        ])

    # Log event
    @Subroutine(TealType.none)
    def log_event():
        msg = Txn.application_args[1]
        return Seq([
            Log(msg),
            Approve(),
        ])

    # Method router
    handle_method = Cond(
        [Txn.application_args[0] == MethodSignature("automate(uint64,uint64,uint64,uint64,uint8,uint64)void"), automate()],
        [Txn.application_args[0] == MethodSignature("checkpoint()void"), checkpoint()],
        [Txn.application_args[0] == MethodSignature("claim_algo()void"), claim_algo()],
        [Txn.application_args[0] == MethodSignature("claim_fpow()void"), claim_fpow()],
        [Txn.application_args[0] == MethodSignature("close()void"), close()],
        [Txn.application_args[0] == MethodSignature("deploy(uint64,uint32)void"), deploy()],
        [Txn.application_args[0] == MethodSignature("log()void"), log_event()],
        [Txn.application_args[0] == MethodSignature("reset()void"), reset()],
        [Txn.application_args[0] == MethodSignature("reload_algo()void"), reload_algo()],
        [Txn.application_args[0] == MethodSignature("deposit(uint64,uint64)void"), deposit()],
        [Txn.application_args[0] == MethodSignature("withdraw(uint64)void"), withdraw()],
        [Txn.application_args[0] == MethodSignature("claim_yield(uint64)void"), claim_yield()],
        [Txn.application_args[0] == MethodSignature("compound_yield()void"), compound_yield()],
        [Txn.application_args[0] == MethodSignature("buyback()void"), buyback()],
        [Txn.application_args[0] == MethodSignature("bury(uint64)void"), bury()],
        [Txn.application_args[0] == MethodSignature("wrap(uint64)void"), wrap()],
        [Txn.application_args[0] == MethodSignature("set_admin(address)void"), set_admin()],
        [Txn.application_args[0] == MethodSignature("new_var(uint64,byte[32],uint64)void"), new_var()],
        [Txn.application_args[0] == MethodSignature("liq()void"), liq()],
    )

    # Main program logic
    program = Cond(
        [Txn.application_id() == Int(0), initialize()],
        [Txn.on_completion() == OnComplete.OptIn, Approve()],
        [Txn.on_completion() == OnComplete.CloseOut, Approve()],
        [Txn.on_completion() == OnComplete.UpdateApplication,
         And(Txn.sender() == App.globalGet(admin_key), Approve())],
        [Txn.on_completion() == OnComplete.DeleteApplication,
         And(Txn.sender() == App.globalGet(admin_key), Approve())],
        [Txn.on_completion() == OnComplete.NoOp, handle_method],
    )

    return program


def clear_state_program():
    """Clear state program - always approves."""
    return Approve()


if __name__ == "__main__":
    import os

    # Compile and output TEAL
    approval_teal = compileTeal(approval_program(), mode=Mode.Application, version=8)
    clear_teal = compileTeal(clear_state_program(), mode=Mode.Application, version=8)

    # Write to files
    with open("approval.teal", "w") as f:
        f.write(approval_teal)

    with open("clear.teal", "w") as f:
        f.write(clear_teal)

    print("Compiled fPOW smart contract to TEAL")
    print(f"Approval program: {len(approval_teal)} bytes")
    print(f"Clear program: {len(clear_teal)} bytes")
