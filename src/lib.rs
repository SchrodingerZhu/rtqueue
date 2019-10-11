pub mod rt_queue {
    use std::rc::Rc;
    use List::*;
    use State::*;
    use std::borrow::Borrow;
    type Ptr<T> = Rc<List<T>>;
    #[derive(Debug, Eq, PartialEq)]
    enum List<T> {
        Nil,
        Cons(Rc<T>, Rc<List<T>>),
    }

    impl<T> List<T> {
        fn cons(x: T, tail: Rc<Self>) -> Self {
            Cons(Rc::new(x), tail.clone())
        }

        fn is_empty(&self) -> bool {
            match self {
                Nil => true,
                _ => false
            }
        }
        fn tail(&self) -> Ptr<T> {
            match self {
                Nil => panic!("call tail with Nil"),
                Cons(_, t) => t.clone()
            }
        }
        fn head(&self) -> Rc<T> {
            match self {
                Nil => panic!("call head with Nil"),
                Cons(h, _) => h.clone()
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum State<T> {
        Empty,
        Reverse(usize, Ptr<T>, Ptr<T>, Ptr<T>, Ptr<T>),
        Concat(usize, Ptr<T>, Ptr<T>),
        Done(Ptr<T>),
    }

    impl <T>  Clone for State<T> {
        fn clone(&self) -> Self {
            match self {
                Empty => Empty,
                Reverse(n,a, b, c, d) =>
                    Reverse(*n, a.clone(), b.clone(), c.clone(), d.clone()),
                Concat(n, a, b) =>
                    Concat(*n, a.clone(), b.clone()),
                Done(t) =>
                    Done(t.clone())
            }
        }
    }

    #[derive(Debug)]
    pub struct RTQ<T>(Ptr<T>, usize, State<T>, Ptr<T>, usize);

    fn cons_<T>(x: Rc<T>, tail: Ptr<T>) -> Ptr<T> {
        Rc::new(Cons(x, tail.clone()))
    }

    impl <T> State<T> {
        fn next(&self) -> Self {
            match self {
                Reverse(n, x_ptr, fp, y_ptr, rp)
                if !x_ptr.is_empty() =>
                    Reverse(n + 1,
                            x_ptr.tail(),
                            cons_(x_ptr.head(), fp.clone()),
                            y_ptr.tail(),
                            cons_(y_ptr.head(), rp.clone())),
                Reverse(n, _, fp, y_ptr, rp)
                if !y_ptr.is_empty() && y_ptr.tail().is_empty() =>
                    Concat(*n, fp.clone(), cons_(y_ptr.head(), rp.clone())),
                Concat(0, _, acc) => Done(acc.clone()),
                Concat(n, x_ptr, acc) =>
                    Concat(n - 1, x_ptr.tail(), cons_(x_ptr.head(), acc.clone())),
                s => s.clone()
            }
        }

        fn abort(&self) -> Self {
            match self {
                Concat(0, _, tail) if !tail.is_empty() =>
                    Done(tail.tail()),
                Concat(n, fp, acc) =>
                    Concat(n - 1, fp.clone(), acc.clone()),
                Reverse(n, f, fp, r, rp) =>
                    Reverse(n - 1, f.clone(), fp.clone(), r.clone(), rp.clone()),
                s => s.clone()
            }
        }
    }

    fn step<T>(f: Ptr<T>, len_f : usize, s: State<T>, r: Ptr<T>, len_r : usize) -> RTQ<T> {
        let s_ = if f.is_empty() {s.next().next()} else {s.next()};
        match s_ {
            Done(f) => RTQ(f, len_f, Empty, r, len_r),
            m => RTQ(f, len_f, m, r, len_r)
        }
    }

    fn balance<T>(f: Ptr<T>, len_f : usize, s: State<T>, r: Ptr<T>, len_r : usize) -> RTQ<T> {
        if len_r <= len_f {
            step(f, len_f, s, r, len_r)
        } else {
            let empty = Rc::new(Nil);
            step(f.clone(), len_f + len_r, Reverse(0, f, empty.clone(), r, empty.clone()), empty, 0)
        }
    }

    impl <T> RTQ<T> {
        pub fn new() -> Self {
            let empty = Rc::new(Nil);
            RTQ(empty.clone(), 0, Empty, empty.clone(), 0)
        }
        pub fn is_empty(&self) -> bool {
            self.1 == 0
        }
        pub fn push(&self, x: T) -> Self {
            balance(self.0.clone(), self.1, self.2.clone(),
                    Rc::new(List::cons(x, self.3.clone())), self.4 + 1)
        }
        pub fn front(&self) -> &T {
            match self.0.borrow() {
                Nil => panic!("try to get an element from an empty queue"),
                Cons(x, _) => x.borrow()
            }
        }
        pub fn pop(&self) -> Self {
            match self {
                RTQ(f, len_f, s, r, len_r) =>
                    balance(f.tail(), len_f - 1, s.abort(), r.clone(), *len_r)
            }
        }
    }

    impl<T> Clone for RTQ<T> {
        fn clone(&self) -> Self {
            match self {
                RTQ(a, b, c,d,e) =>
                    RTQ(a.clone(), *b, c.clone(), d.clone(), e.clone())
            }
        }
    }

    #[test]
    fn small_case() {
        let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 196883];
        let mut vp = Vec::new();
        let mut q = RTQ::new();
        for i in &v {
            q = q.push(*i);
        }
        while !q.is_empty() {
            vp.push(*q.front());
            q = q.pop();
        }
        assert_eq!(v, vp);
    }

    #[test]
    fn random_case() {
        use rand::prelude::*;
        for _ in 0..20 {
            let mut v : Vec<u128> = Vec::new();
            for _ in 0..random::<u128>() % 10001 {
                v.push(random());
            }
            let mut vp : Vec<u128> = Vec::new();
            let mut q : RTQ<u128> = RTQ::new();
            for i in &v {
                q = q.push(*i);
            }
            print!("");
            while !q.is_empty() {
                vp.push(*q.front());
                q = q.pop();
            }
            assert_eq!(v, vp);
        }
    }

    #[test]
    fn versioned() {
        use rand::prelude::*;
        let mut history : Vec<Vec<u128>> = vec![Vec::new()];
        let mut history_v = vec![RTQ::new()];
        let mut version = 0;
        for _ in 0..20 {
            let mut q = history_v[version].clone();
            let mut vec = history[version].clone();
            for _ in 0..random::<u128>() % 500 {
                vec.push(random());
                q = q.push(*vec.last().unwrap());
            }
            history_v.push(q);
            history.push(vec);
            version += 1;
        }
        for i in 1..=20 {
            let mut q = history_v[i].clone();
            let mut vec = history[i].clone();
            vec.reverse();
            for _ in 0..random::<usize>() % vec.len() {
                vec.pop();
                q = q.pop();
            }
            vec.reverse();
            history_v.push(q);
            history.push(vec);
        }
        for i in 0..40 {
            let mut vp : Vec<u128> = Vec::new();
            let mut q = history_v[i].clone();
            while !q.is_empty() {
                vp.push(*q.front());
                q = q.pop();
            }
            assert_eq!(history[i], vp);
        }
    }

}