# Task Previous Complete Fields Implementation Plan

> **For Claude:** Use `${SUPERPOWERS_SKILLS_ROOT}/skills/collaboration/executing-plans/SKILL.md` to implement this plan task-by-task.

**Goal:** Make `task.previous` always contain all 8 constraint fields (cpu, memory, container, gpu, fpga, disks, max_retries, return_codes) with None values for unset fields.

**Architecture:** Update the `TASK_PREVIOUS_TYPE` struct definition in the analysis layer to include max_retries and return_codes fields. The Rust struct constructor automatically sets unprovided fields to None, so no engine changes needed.

**Tech Stack:** Rust, WDL analysis (wdl-analysis crate), WDL AST constants

---

## Task 1: Add TASK_FIELD constants for max_retries and return_codes

**Context:** The analysis layer needs field name constants to reference in the struct definition. These already exist as TASK_REQUIREMENT_* constants but we need TASK_FIELD_* versions for consistency with other task fields.

**Files:**
- Modify: `crates/wdl-ast/src/v1/task.rs:256` (after TASK_FIELD_EXT)

**Step 1: Add max_retries field constant**

Add after line 256 (after `TASK_FIELD_EXT`):

```rust
/// The name of the task's `max_retries` field.
pub const TASK_FIELD_MAX_RETRIES: &str = "max_retries";
```

**Step 2: Add return_codes field constant**

Add after the max_retries constant:

```rust
/// The name of the task's `return_codes` field.
pub const TASK_FIELD_RETURN_CODES: &str = "return_codes";
```

**Step 3: Verify constants are exported**

Run: `cargo build --package wdl-ast`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add crates/wdl-ast/src/v1/task.rs
git commit -m "feat: add TASK_FIELD constants for max_retries and return_codes"
```

---

## Task 2: Update TASK_PREVIOUS_TYPE to include all 8 constraint fields

**Context:** The `TASK_PREVIOUS_TYPE` struct definition currently has 6 fields. We need to add max_retries and return_codes so that `task.previous` always has all constraint fields available, with None for unset values.

**Files:**
- Modify: `crates/wdl-analysis/src/types/v1.rs:200-234` (TASK_PREVIOUS_TYPE definition)

**Step 1: Import new field constants**

Check if TASK_FIELD_MAX_RETRIES and TASK_FIELD_RETURN_CODES are imported. If not, add to imports section around line 115:

```rust
use wdl_ast::v1::{
    // ... existing imports
    TASK_FIELD_MAX_RETRIES,
    TASK_FIELD_RETURN_CODES,
};
```

**Step 2: Add max_retries field to struct**

In the `TASK_PREVIOUS_TYPE` definition (around line 223, after TASK_FIELD_DISKS), add:

```rust
(
    TASK_FIELD_MAX_RETRIES,
    Type::from(PrimitiveType::Integer).optional(),
),
```

**Step 3: Add return_codes field to struct**

After max_retries, add return_codes with union type matching RETURN_CODES_TYPES (Int | String | Array[Int]):

```rust
(
    TASK_FIELD_RETURN_CODES,
    Type::Union.optional(),
),
```

Note: Using Type::Union because return_codes can be Int, String, or Array[Int]. The optional() makes it Int? | String? | Array[Int]? | None.

**Step 4: Verify compilation**

Run: `cargo build --package wdl-analysis`
Expected: Successful compilation

**Step 5: Commit**

```bash
git add crates/wdl-analysis/src/types/v1.rs
git commit -m "feat: add max_retries and return_codes to task.previous struct"
```

---

## Task 3: Run all tests to verify backward compatibility

**Context:** Since we only added fields to the struct (and Rust struct constructor sets unprovided fields to None automatically), existing tests should pass unchanged. The engine's `get_previous_field()` already uses the struct type from analysis, so it automatically gets the new fields.

**Files:**
- No changes, verification only

**Step 1: Run analysis tests**

Run: `cargo test --package wdl-analysis --test analysis --features codespan`
Expected: All 92+ tests pass

**Step 2: Run engine tests**

Run: `cargo test --package wdl-engine --test tasks --features codespan-reporting`
Expected: All 84+ tests pass

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 4: Verify no regressions**

Check that:
- Existing tests accessing `task.previous.memory`, `task.previous.cpu`, etc. still work
- No blessed outputs changed unexpectedly
- All error.txt files are unchanged (analysis behavior is backward compatible)

**Step 5: Commit if any test outputs changed**

If BLESS=1 was needed to update any test outputs:

```bash
git add crates/wdl-engine/tests/
git commit -m "test: update blessed outputs for task.previous struct changes"
```

---

## Task 4: Optional - Create test demonstrating new fields

**Context:** While not strictly necessary (the change is backward compatible), we could add a test showing that max_retries and return_codes are now accessible via task.previous.

**Files:**
- Create: `crates/wdl-engine/tests/tasks/task-previous-complete-fields/source.wdl`
- Create: `crates/wdl-engine/tests/tasks/task-previous-complete-fields/inputs.json`

**Step 1: Create test WDL file**

```wdl
version 1.3

task test_previous_complete {
  requirements {
    memory: if task.attempt > 0 then select_first([task.previous.memory, 0]) * 2 else 128000000
    max_retries: 2
    return_codes: 0
  }

  command <<<
    # Fail on first attempt to trigger retry
    if [ ~{task.attempt} -lt 1 ]; then
      echo "Attempt ~{task.attempt}: max_retries=~{select_first([task.previous.max_retries, -1])}"
      exit 1
    else
      echo "Attempt ~{task.attempt}: max_retries=~{select_first([task.previous.max_retries, -1])}"
      echo "SUCCESS"
      exit 0
    fi
  >>>

  output {
    Int final_attempt = task.attempt
    Int? previous_max_retries = task.previous.max_retries
  }
}
```

**Step 2: Create inputs file**

```json
{}
```

**Step 3: Run test with BLESS**

Run: `BLESS=1 cargo test --package wdl-engine --test tasks --features codespan-reporting task-previous-complete-fields_local`
Expected: Test passes and generates blessed outputs

**Step 4: Verify test output**

Check that stdout shows:
- Attempt 0 with max_retries=-1 (None, so select_first returns -1)
- Attempt 1 with max_retries=2 (from previous attempt)

**Step 5: Commit**

```bash
git add crates/wdl-engine/tests/tasks/task-previous-complete-fields/
git commit -m "test: add test for task.previous with max_retries and return_codes fields"
```

---

## Verification Checklist

Before marking complete:

- [ ] TASK_FIELD_MAX_RETRIES constant added to wdl-ast
- [ ] TASK_FIELD_RETURN_CODES constant added to wdl-ast
- [ ] TASK_PREVIOUS_TYPE includes max_retries field (Int?)
- [ ] TASK_PREVIOUS_TYPE includes return_codes field (Union?)
- [ ] All analysis tests pass
- [ ] All engine tests pass
- [ ] Full test suite passes
- [ ] Optional test created and passing
- [ ] All changes committed

## Notes

- The struct constructor automatically sets unprovided fields to None, so no engine code changes needed
- Using Type::Union for return_codes because it can be Int | String | Array[Int]
- This is backward compatible - existing sparse field access still works
- New WDL code can now access task.previous.max_retries and task.previous.return_codes
