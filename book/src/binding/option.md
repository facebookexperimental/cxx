{{#title rust::Option<T> — Rust ♡ C++}}
# rust::Option\<T\>

### Public API:

```cpp,hidelines=...
// rust/cxx.h
...
...namespace rust {

template <typename T>
class Option final {};

template <typename T>
class Option<T&> {
public:
  Option() noexcept;
  Option(Option&&) noexcept;
  Option(T&) noexcept;
  ~Option() noexcept;

  Option<T&>& operator=(Option&&) noexcept;
  const T& operator*() const noexcept;
  const T* operator->() const noexcept;
  T& operator*() noexcept;
  T* operator->() noexcept;

  bool has_value() const noexcept;
  const T& value() const noexcept;
  T& value() noexcept;
  void reset();
  void set(T&) noexcept;
  static Option<T&> from_raw(T*) noexcept;
  T* into_raw() noexcept;
};

template <typename T>
class Option<const T&> {
public:
  Option() noexcept;
  Option(const Option&) noexcept;
  Option(Option&&) noexcept;
  Option(const T&) noexcept;
  ~Option() noexcept;

  Option<const T&>& operator=(Option&&) noexcept;
  const T& operator*() const noexcept;
  const T* operator->() const noexcept;

  bool has_value() const noexcept;
  const T& value() const noexcept;
  void reset();
  void set(const T&) noexcept;
  static Option<const T&> from_raw(const T*) noexcept;
  const T* into_raw() const noexcept;
};

template <typename T>
class Option<Box<T>> {
public:
  Option() noexcept;
  Option(Option&&) noexcept;
  Option(Box<T>&&) noexcept;
  ~Option() noexcept;

  Option<Box<T>>& operator=(Option&&) noexcept;
  const Box<T>& operator*() const noexcept;
  const Box<T>* operator->() const noexcept;
  Box<T>& operator*() noexcept;
  Box<T>* operator->() noexcept;

  bool has_value() const noexcept;
  const Box<T>& value() const noexcept;
  Box<T>& value() noexcept;
  void reset();
  void set(Box<T>) noexcept;

  // Important: requires that `raw` came from an into_raw call. Do not pass a
  // pointer from `new` or any other source.
  static Option<Box<T>> from_raw(T*) noexcept;
  T* into_raw() noexcept;
};
...} // namespace rust
```

### Restrictions:

Option<T> only supports pointer-sized references and Box-es; that is, no
fat pointers like &str (though &String is supported) or Box<[u8]>. Also,
you can only pass Options themselves by value. &Option<T> is not allowed.

## Example

```rust,noplayground
// src/main.rs

#[cxx::bridge]
mod ffi {
    struct Shared {
        v: u32,
    }

    unsafe extern "C++" {
        include!("example/include/example.h");

        fn f(elements: Option<&Shared>);
    }
}

fn main() {
    let shared = Shared { v: 3 };
    ffi::f(Some(&shared));
    ffi::f(None);
}
```

```cpp
// include/example.h

#pragma once
#include "example/src/main.rs.h"
#include "rust/cxx.h"

void f(rust::Option<const Shared&>);
```

```cpp
// src/example.cc

#include "example/include/example.h"

void f(rust::Option<const Shared&> o) {
  if (o.has_value()) {
    // Pointer is guaranteed to be non-null
    std::cout << shared.value().v << std::endl;
  }
}
```
