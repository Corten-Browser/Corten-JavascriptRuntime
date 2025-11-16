//! Tests for Opcode enum
//! TDD: RED phase - these tests should fail initially

use bytecode_system::{Opcode, RegisterId};

#[test]
fn test_load_constant_opcode() {
    let op = Opcode::LoadConstant(0);
    match op {
        Opcode::LoadConstant(idx) => assert_eq!(idx, 0),
        _ => panic!("Expected LoadConstant"),
    }
}

#[test]
fn test_load_undefined_opcode() {
    let op = Opcode::LoadUndefined;
    assert!(matches!(op, Opcode::LoadUndefined));
}

#[test]
fn test_load_null_opcode() {
    let op = Opcode::LoadNull;
    assert!(matches!(op, Opcode::LoadNull));
}

#[test]
fn test_load_true_opcode() {
    let op = Opcode::LoadTrue;
    assert!(matches!(op, Opcode::LoadTrue));
}

#[test]
fn test_load_false_opcode() {
    let op = Opcode::LoadFalse;
    assert!(matches!(op, Opcode::LoadFalse));
}

#[test]
fn test_load_global_opcode() {
    let op = Opcode::LoadGlobal("console".to_string());
    match op {
        Opcode::LoadGlobal(name) => assert_eq!(name, "console"),
        _ => panic!("Expected LoadGlobal"),
    }
}

#[test]
fn test_store_global_opcode() {
    let op = Opcode::StoreGlobal("myVar".to_string());
    match op {
        Opcode::StoreGlobal(name) => assert_eq!(name, "myVar"),
        _ => panic!("Expected StoreGlobal"),
    }
}

#[test]
fn test_load_local_opcode() {
    let reg = RegisterId(5);
    let op = Opcode::LoadLocal(reg);
    match op {
        Opcode::LoadLocal(r) => assert_eq!(r.0, 5),
        _ => panic!("Expected LoadLocal"),
    }
}

#[test]
fn test_store_local_opcode() {
    let reg = RegisterId(10);
    let op = Opcode::StoreLocal(reg);
    match op {
        Opcode::StoreLocal(r) => assert_eq!(r.0, 10),
        _ => panic!("Expected StoreLocal"),
    }
}

#[test]
fn test_arithmetic_opcodes() {
    assert!(matches!(Opcode::Add, Opcode::Add));
    assert!(matches!(Opcode::Sub, Opcode::Sub));
    assert!(matches!(Opcode::Mul, Opcode::Mul));
    assert!(matches!(Opcode::Div, Opcode::Div));
    assert!(matches!(Opcode::Mod, Opcode::Mod));
    assert!(matches!(Opcode::Neg, Opcode::Neg));
}

#[test]
fn test_comparison_opcodes() {
    assert!(matches!(Opcode::Equal, Opcode::Equal));
    assert!(matches!(Opcode::StrictEqual, Opcode::StrictEqual));
    assert!(matches!(Opcode::NotEqual, Opcode::NotEqual));
    assert!(matches!(Opcode::StrictNotEqual, Opcode::StrictNotEqual));
    assert!(matches!(Opcode::LessThan, Opcode::LessThan));
    assert!(matches!(Opcode::LessThanEqual, Opcode::LessThanEqual));
    assert!(matches!(Opcode::GreaterThan, Opcode::GreaterThan));
    assert!(matches!(Opcode::GreaterThanEqual, Opcode::GreaterThanEqual));
}

#[test]
fn test_jump_opcode() {
    let op = Opcode::Jump(42);
    match op {
        Opcode::Jump(offset) => assert_eq!(offset, 42),
        _ => panic!("Expected Jump"),
    }
}

#[test]
fn test_jump_if_true_opcode() {
    let op = Opcode::JumpIfTrue(100);
    match op {
        Opcode::JumpIfTrue(offset) => assert_eq!(offset, 100),
        _ => panic!("Expected JumpIfTrue"),
    }
}

#[test]
fn test_jump_if_false_opcode() {
    let op = Opcode::JumpIfFalse(200);
    match op {
        Opcode::JumpIfFalse(offset) => assert_eq!(offset, 200),
        _ => panic!("Expected JumpIfFalse"),
    }
}

#[test]
fn test_return_opcode() {
    assert!(matches!(Opcode::Return, Opcode::Return));
}

#[test]
fn test_create_object_opcode() {
    assert!(matches!(Opcode::CreateObject, Opcode::CreateObject));
}

#[test]
fn test_load_property_opcode() {
    let op = Opcode::LoadProperty("name".to_string());
    match op {
        Opcode::LoadProperty(prop) => assert_eq!(prop, "name"),
        _ => panic!("Expected LoadProperty"),
    }
}

#[test]
fn test_store_property_opcode() {
    let op = Opcode::StoreProperty("value".to_string());
    match op {
        Opcode::StoreProperty(prop) => assert_eq!(prop, "value"),
        _ => panic!("Expected StoreProperty"),
    }
}

#[test]
fn test_create_closure_opcode() {
    let op = Opcode::CreateClosure(3);
    match op {
        Opcode::CreateClosure(idx) => assert_eq!(idx, 3),
        _ => panic!("Expected CreateClosure"),
    }
}

#[test]
fn test_call_opcode() {
    let op = Opcode::Call(5);
    match op {
        Opcode::Call(argc) => assert_eq!(argc, 5),
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_opcode_clone() {
    let op1 = Opcode::LoadConstant(42);
    let op2 = op1.clone();
    match (op1, op2) {
        (Opcode::LoadConstant(a), Opcode::LoadConstant(b)) => assert_eq!(a, b),
        _ => panic!("Clone failed"),
    }
}

#[test]
fn test_opcode_debug() {
    let op = Opcode::Add;
    let debug_str = format!("{:?}", op);
    assert!(debug_str.contains("Add"));
}

#[test]
fn test_register_id_creation() {
    let reg = RegisterId(100);
    assert_eq!(reg.0, 100);
}

#[test]
fn test_register_id_clone() {
    let reg1 = RegisterId(25);
    let reg2 = reg1.clone();
    assert_eq!(reg1.0, reg2.0);
}

#[test]
fn test_register_id_debug() {
    let reg = RegisterId(42);
    let debug_str = format!("{:?}", reg);
    assert!(debug_str.contains("42"));
}

#[test]
fn test_opcode_equality() {
    assert_eq!(Opcode::Add, Opcode::Add);
    assert_eq!(Opcode::LoadConstant(1), Opcode::LoadConstant(1));
    assert_ne!(Opcode::LoadConstant(1), Opcode::LoadConstant(2));
}
