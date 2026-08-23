use egui_nav_stack::{NavAction, NavStack};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Screen {
    Home,
    Catalog,
    Detail(u32),
    Settings,
}

#[test]
fn test_stack_new_and_empty() {
    let stack_root = NavStack::new(Screen::Home);
    assert_eq!(stack_root.len(), 1);
    assert_eq!(stack_root.top(), Some(&Screen::Home));
    assert!(!stack_root.is_empty());
    assert!(!stack_root.can_pop());

    let stack_empty: NavStack<Screen> = NavStack::empty();
    assert_eq!(stack_empty.len(), 0);
    assert_eq!(stack_empty.top(), None);
    assert!(stack_empty.is_empty());
    assert!(!stack_empty.can_pop());
}

#[test]
fn test_push_pop_sequence() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);
    stack.push(Screen::Detail(101));

    assert_eq!(stack.len(), 3);
    assert!(stack.can_pop());
    assert_eq!(stack.top(), Some(&Screen::Detail(101)));

    let popped1 = stack.pop();
    assert_eq!(popped1, Some(Screen::Detail(101)));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.top(), Some(&Screen::Catalog));

    let popped2 = stack.pop();
    assert_eq!(popped2, Some(Screen::Catalog));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.top(), Some(&Screen::Home));

    let popped_root = stack.pop();
    assert_eq!(popped_root, Some(Screen::Home));
    assert!(stack.is_empty());
    assert_eq!(stack.top(), None);

    // Popping empty stack returns None without panic
    let popped_none = stack.pop();
    assert_eq!(popped_none, None);
}

#[test]
fn test_undo_redo_history() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);
    stack.push(Screen::Detail(1));
    stack.push(Screen::Detail(2));

    assert_eq!(stack.len(), 4);
    assert!(!stack.can_go_forward());
    assert_eq!(stack.forward_len(), 0);

    // 1. Navigate Back (Undo)
    let back1 = stack.go_back();
    assert_eq!(back1, Some(Screen::Detail(2)));
    assert_eq!(stack.top(), Some(&Screen::Detail(1)));
    assert!(stack.can_go_forward());
    assert_eq!(stack.forward_len(), 1);

    // 2. Navigate Back again (Undo)
    let back2 = stack.go_back();
    assert_eq!(back2, Some(Screen::Detail(1)));
    assert_eq!(stack.top(), Some(&Screen::Catalog));
    assert_eq!(stack.forward_len(), 2);

    // 3. Navigate Forward (Redo)
    let fwd1 = stack.go_forward();
    assert_eq!(fwd1, Some(Screen::Detail(1)));
    assert_eq!(stack.top(), Some(&Screen::Detail(1)));
    assert_eq!(stack.forward_len(), 1);

    // 4. Navigate Forward again (Redo)
    let fwd2 = stack.go_forward();
    assert_eq!(fwd2, Some(Screen::Detail(2)));
    assert_eq!(stack.top(), Some(&Screen::Detail(2)));
    assert!(!stack.can_go_forward());

    // 5. Redo on empty forward history returns None
    assert_eq!(stack.go_forward(), None);
}

#[test]
fn test_branching_push_clears_redo() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);
    stack.push(Screen::Detail(1));

    // Go back to Catalog
    stack.go_back();
    assert!(stack.can_go_forward());
    assert_eq!(stack.forward_len(), 1);

    // Pushing a new screen branches history and clears forward redo stack
    stack.push(Screen::Settings);
    assert!(!stack.can_go_forward());
    assert_eq!(stack.forward_len(), 0);
    assert_eq!(stack.entries(), &[Screen::Home, Screen::Catalog, Screen::Settings]);
}

#[test]
fn test_replace_top() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);

    let prev = stack.replace_top(Screen::Settings);
    assert_eq!(prev, Some(Screen::Catalog));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.top(), Some(&Screen::Settings));
    assert_eq!(stack.entries(), &[Screen::Home, Screen::Settings]);

    // Replace top on empty stack pushes as root
    let mut empty_stack: NavStack<Screen> = NavStack::empty();
    let prev_empty = empty_stack.replace_top(Screen::Home);
    assert_eq!(prev_empty, None);
    assert_eq!(empty_stack.len(), 1);
    assert_eq!(empty_stack.top(), Some(&Screen::Home));
}

#[test]
fn test_pop_to() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);
    stack.push(Screen::Detail(1));
    stack.push(Screen::Detail(2));
    stack.push(Screen::Settings);

    // Pop back to Catalog
    let popped = stack.pop_to(|s| matches!(s, Screen::Catalog));
    assert_eq!(popped, vec![Screen::Settings, Screen::Detail(2), Screen::Detail(1)]);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.top(), Some(&Screen::Catalog));

    // Popped items are stored in forward history
    assert_eq!(stack.forward_len(), 3);
    assert_eq!(stack.go_forward(), Some(Screen::Detail(1)));

    // Pop to non-existent screen returns empty vec and leaves stack unchanged
    let non_existent_popped = stack.pop_to(|s| matches!(s, Screen::Detail(999)));
    assert!(non_existent_popped.is_empty());
}

#[test]
fn test_pop_to_root() {
    let mut stack = NavStack::new(Screen::Home);
    stack.push(Screen::Catalog);
    stack.push(Screen::Detail(42));
    stack.push(Screen::Settings);

    let popped = stack.pop_to_root();
    assert_eq!(popped, vec![Screen::Settings, Screen::Detail(42), Screen::Catalog]);
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.top(), Some(&Screen::Home));

    // Pop to root on 1-element stack is a no-op
    let popped_single = stack.pop_to_root();
    assert!(popped_single.is_empty());
    assert_eq!(stack.len(), 1);

    // Pop to root on empty stack is a no-op
    let mut empty_stack: NavStack<Screen> = NavStack::empty();
    let popped_empty = empty_stack.pop_to_root();
    assert!(popped_empty.is_empty());
    assert!(empty_stack.is_empty());
}

#[test]
fn test_apply_nav_actions() {
    let mut stack = NavStack::new(Screen::Home);

    stack.apply(NavAction::Push(Screen::Catalog));
    assert_eq!(stack.entries(), &[Screen::Home, Screen::Catalog]);

    stack.apply(NavAction::ReplaceTop(Screen::Settings));
    assert_eq!(stack.entries(), &[Screen::Home, Screen::Settings]);

    stack.apply(NavAction::Push(Screen::Detail(5)));
    assert_eq!(stack.len(), 3);

    // Test Action::Pop and Action::Forward
    stack.apply(NavAction::Pop);
    assert_eq!(stack.top(), Some(&Screen::Settings));
    assert!(stack.can_go_forward());

    stack.apply(NavAction::Forward);
    assert_eq!(stack.top(), Some(&Screen::Detail(5)));

    stack.apply(NavAction::PopToRoot);
    assert_eq!(stack.entries(), &[Screen::Home]);

    stack.apply(NavAction::Pop);
    assert!(stack.is_empty());
}

#[test]
fn test_top_mut_and_iteration() {
    #[derive(Clone, Debug, PartialEq)]
    struct CounterScreen {
        count: i32,
    }

    let mut stack = NavStack::new(CounterScreen { count: 0 });
    stack.push(CounterScreen { count: 10 });

    if let Some(top_screen) = stack.top_mut() {
        top_screen.count += 5;
    }

    assert_eq!(stack.top(), Some(&CounterScreen { count: 15 }));

    let counts: Vec<i32> = stack.iter().map(|s| s.count).collect();
    assert_eq!(counts, vec![0, 15]);
}
