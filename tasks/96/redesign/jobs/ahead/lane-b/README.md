# Tests ahead of the compiler: lane b (the C interface)

Each test here follows the spec and waits for the part of the compiler it needs. It moves back to
`tests/conformance/<RULE>/` in the job that builds that part.

- `GRM-35/accept_typed_parameters_member_refs_and_class_sections.em` (W3, `c03d4557`; moved
  2026-11-13): an overlay entry's parameters as the header gives them. `ember check --syntax-only`
  refuses `owns tex: Texture` (E0005) and `s: *c_char` (E0100, "expected a contract word"). It looked
  green because the quick check skipped `parse-pass` tests; it checks them now.
