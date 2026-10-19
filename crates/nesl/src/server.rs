
    #[test]
    fn compile_response_includes_mapper_type() {
        let small = r#"{"method":"compile","src":"fn main() {}"}"#;
        let res: Response = serde_json::from_str(&handle(small)).unwrap();
        if let Response::CompileSuccess { mapper, .. } = res { assert_eq!(mapper, 0); } else { panic!(); }
        
        let big = format!(r#"{{"method":"compile","src":"{}fn main() {{}}"}}"#,
            (0..9).map(|i| format!("map M{i} = [{}];", (0..128).map(|_| format!("\"{}\"", ".".repeat(30))).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join("\n"));
        let res: Response = serde_json::from_str(&handle(&big)).unwrap();
        if let Response::CompileSuccess { mapper, .. } = res { assert_eq!(mapper, 2); } else { panic!(); }
    }
}
