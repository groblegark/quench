// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Unit tests for inline test detection via git diff.

use super::*;

#[test]
fn changes_in_cfg_test_detects_test_additions() {
    let diff = r#"diff --git a/src/parser.rs b/src/parser.rs
index abc123..def456 100644
--- a/src/parser.rs
+++ b/src/parser.rs
@@ -1,3 +1,15 @@
 pub fn parse() -> bool {
     true
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_parse() {
+        assert!(parse());
+    }
+}
"#;

    assert!(changes_in_cfg_test(diff));
}

#[test]
fn changes_in_cfg_test_false_for_non_test_changes() {
    let diff = r#"diff --git a/src/parser.rs b/src/parser.rs
index abc123..def456 100644
--- a/src/parser.rs
+++ b/src/parser.rs
@@ -1,3 +1,4 @@
 pub fn parse() -> bool {
-    true
+    // Updated implementation
+    false
 }
"#;

    assert!(!changes_in_cfg_test(diff));
}

#[test]
fn changes_in_cfg_test_tracks_brace_depth() {
    let diff = r#"diff --git a/src/parser.rs b/src/parser.rs
--- a/src/parser.rs
+++ b/src/parser.rs
@@ -1,5 +1,12 @@
 pub fn parse() -> bool { true }

 #[cfg(test)]
 mod tests {
+    use super::*;
+
+    #[test]
+    fn nested() {
+        assert!(true);
+    }
 }
"#;

    assert!(changes_in_cfg_test(diff));
}

#[test]
fn changes_in_cfg_test_empty_diff() {
    assert!(!changes_in_cfg_test(""));
}

#[test]
fn changes_in_cfg_test_context_only() {
    // Context lines (prefixed with space) shouldn't count as changes
    let diff = r#"diff --git a/src/parser.rs b/src/parser.rs
--- a/src/parser.rs
+++ b/src/parser.rs
@@ -1,5 +1,5 @@
 pub fn parse() -> bool { true }

 #[cfg(test)]
 mod tests {
     fn test_parse() { }
 }
"#;

    assert!(!changes_in_cfg_test(diff));
}
