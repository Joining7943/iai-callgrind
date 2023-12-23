//! The public interface to valgrind's client request mechanism
//!
//! You can use these macros to manipulate and query Valgrind's execution inside your own programs.
//!
//! # Organization
//!
//! The client requests are organized into modules representing the source header file. So, if you
//! search for a client request originating from the `valgrind.h` header file, the client request
//! can be found in the [`crate::client_requests::valgrind`] module. Instead of using macros like in
//! valgrind we're using functions and small letter names, stripping the prefix if it is equal to
//! the enclosing module. For example the client request `RUNNING_ON_VALGRIND` from the `valgrind.h`
//! file equals [`crate::client_requests::valgrind::running_on_valgrind`] and
//! `VALGRIND_COUNT_ERRORS` from the same `valgrind.h` header file equals
//! [`crate::client_requests::valgrind::count_errors`].
//!
//! # Features
//!
//! The client requests are organized into two separate features. The `client_requests_defs` feature
//! enables the definitions but doesn't run any code yet. To actually run the client requests you
//! need to enable the `client_requests` feature. The `client_requests` feature implies
//! `client_requests_defs`. For example, if you need to include the client requests into your
//! production code, you usually don't want them to run if not running under valgrind in the
//! `iai-callgrind` benchmarks. In order to achieve this, the `client_requests_defs` can be safely
//! included in the production code since the compiler will optimize them completely away. So, in
//! your `Cargo.toml` file, you can do
//!
//! ```toml
//! [dependencies]
//! iai-callgrind = { version = "0.9.0", default-features = false, features = [
//!     "client_requests_defs"
//! ]}
//!
//! [dev-dependencies]
//! iai-callgrind = { version = "0.9.0", features = ["client_requests"] }
//! ```
//!
//! # Performance
//!
//! Depending on the architecture you are making use of the client requests, the client requests are
//! optimized to run with the same overhead like the original client requests. The
//! optimizations are based on inline assembly with the `asm!` macro and depend on the availability
//! of it on a specific architecture/target. Since inline assembly is not stable on all
//! architectures which are supported by valgrind, we cannot provide optimized client requests for
//! them. But, you can still use the non-optimized version on all platforms which would be supported
//! by valgrind. In the end, all platforms which are covered by valgrind are also covered by
//! `iai-callgrind`.
//!
//! The non-optimized version add overhead because we need to wrap the macro from the header file in
//! a function call. This additional function call equals the additional overhead compared to the
//! original valgrind implementation. Although this is usually not much, we try hard to avoid any
//! overhead to not slow down the benchmarks.
//!
//! Here's a short overview on which targets the optimized client requests are available
//!
//! | Target                | Optimized | Reason  |
//! | --------------------- | --------- | ------- |
//! | `x86_64/linux`        | yes | -
//! | `x86_64/freebsd`      | yes | -
//! | `x86_64/apple+darwin` | yes | -
//! | `x86_64/windows+gnu`  | yes | -
//! | `x86_64/solaris`      | yes | -
//! | `x86/linux`           | yes | -
//! | `x86/freebsd`         | yes | -
//! | `x86/apple+darwin`    | yes | -
//! | `x86/windows+gnu`     | yes | -
//! | `x86/solaris`         | yes | -
//! | `x86/windows+msvc`    | no  | TODO
//! | `arm/linux`           | no  | TODO
//! | `aarch64/linux`       | no  | TODO
//! | `x86_64/windows+msvc` | no  | unsupported by valgrind
//! | `s390x/linux`         | no  | unstable inline assembly
//! | `mips32/linux`        | no  | unstable inline assembly
//! | `mips64/linux`        | no  | unstable inline assembly
//! | `powerpc/linux`       | no  | unstable inline assembly
//! | `powerpc64/linux`     | no  | unstable inline assembly
//! | `powerpc64le/linux`   | no  | unstable inline assembly
//! | `nanomips/linux`      | no  | unstable inline assembly
//!
//! All other platforms you don't find in the table above are also not supported by valgrind, yet.
//!
//! # Sources and additional documentation
//!
//! A lot of the library documentation of the client requests within this module and its submodules
//! are taken from the online manual and the valgrind header files. For more details see also [The
//! Client Request
//! mechanism](https://valgrind.org/docs/manual/manual-core-adv.html#manual-core-adv.clientreq)

#![allow(clippy::inline_always)]

/// Return true if a client request is defined and available in the used valgrind version
///
/// We do this check to avoid incompatibilities with older valgrinds version which might not have
/// all client requests available we're offering.
///
/// We're only using constant values known at compile time, which the compiler will finally optimize
/// away, so this macro costs us nothing.
macro_rules! is_def {
    ($user_req:path) => {{ $user_req as cty::c_uint > 0x1000 }};
}

/// The macro which uses [`valgrind_do_client_request_stmt`] or [`valgrind_do_client_request_expr`]
/// to execute the client request.
///
/// For internal use only!
///
/// This macro has two forms: The first takes 7 arguments `name, request, arg1, ..., arg5` and
/// returns `()`. The expanded macro of this form calls [`valgrind_do_client_request_stmt`]. The
/// second first has 8 arguments `name, default, request, arg1, ..., arg5` returning a `usize`. The
/// expanded macro of this form calls [`valgrind_do_client_request_expr`].
///
/// Both forms will raise a [`fatal_error`] in case the [`is_def`] macro returns false.
macro_rules! do_client_request {
    ($name:literal, $request:path, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {{
        if is_def!($request) {
            valgrind_do_client_request_stmt(
                $request as cty::c_uint,
                $arg1,
                $arg2,
                $arg3,
                $arg4,
                $arg5,
            );
        } else {
            fatal_error($name);
        }
    }};
    (
        $name:literal,
        $default:expr,
        $request:path,
        $arg1:expr,
        $arg2:expr,
        $arg3:expr,
        $arg4:expr,
        $arg5:expr
    ) => {{
        if is_def!($request) {
            valgrind_do_client_request_expr(
                $default,
                $request as cty::c_uint,
                $arg1,
                $arg2,
                $arg3,
                $arg4,
                $arg5,
            )
        } else {
            fatal_error($name);
        }
    }};
}

/// Convenience macro to create a `\0`-byte terminated [`std::ffi::CString`] from a literal string
///
/// The string literal passed to this macro must contain or end with a `\0`-byte. If you need a
/// checked version of [`std::ffi::CString`] you can use [`std::ffi::CString::new`].
///
/// # Safety
///
/// This macro is unsafe but convenient and very efficient. It is your responsibility to ensure that
/// the input string literal does not contain any `\0` bytes.
#[macro_export]
macro_rules! cstring {
    ($string:literal) => {{ std::ffi::CString::from_vec_with_nul_unchecked(concat!($string, "\0").as_bytes().to_vec()) }};
}

/// Convenience macro to create a `\0`-byte terminated [`std::ffi::CString`] from a format string
///
/// The format string passed to this macro must not contain or end with a `\0`-byte.
///
/// # Safety
///
/// The same safety conditions as to the [`cstring`] macro apply here
#[macro_export]
macro_rules! format_cstring {
    ($($args:tt)*) => {{ std::ffi::CString::from_vec_with_nul_unchecked(format!("{}\0", format_args!($($args)*)).into_bytes()) }};
}

cfg_if! {
    if #[cfg(feature = "client_requests")] {
        /// Allow prints to valgrind log
        ///
        /// This macro is a safe variant of the `VALGRIND_PRINTF` function, checking for `\0` bytes in the
        /// formatting string. However, if you're sure there are no `\0` bytes present you can
        /// safely use [`crate::valgrind_printf_unchecked`] which performs better compared to this
        /// macro and should perform around equal to the original `VALGRIND_PRINTF` function from
        /// the `valgrind.h` header file.
        #[macro_export]
        macro_rules! valgrind_printf {
            ($($args:tt)*) => {{
                match std::ffi::CString::from_vec_with_nul(
                    format!("{}\0", format_args!($($args)*)).into_bytes()
                ) {
                    Ok(c_string) => {
                        unsafe {
                            $crate::client_requests::__valgrind_print(
                                c_string.as_ptr() as *const ()
                            );
                        }
                        Ok(())
                    },
                    Err(error) => Err(
                        $crate::client_requests::error::ClientRequestError::from(error)
                    )
                }
            }};
        }

        /// Allow prints to valgrind log
        ///
        /// Use this macro only if you are sure there are no `\0`-bytes in the formatted string. If
        /// unsure use the safe [`crate::valgrind_printf`] variant.
        ///
        /// This variant performs better than [`crate::valgrind_printf`] and  should perform around
        /// equal to the original `VALGRIND_PRINTF` function from the `valgrind.h` header file.
        #[macro_export]
        macro_rules! valgrind_printf_unchecked {
            ($($args:tt)*) => {{
                let string = format!("{}\0", format_args!($($args)*));
                $crate::client_requests::__valgrind_print(string.as_ptr() as *const ());
            }};
        }

        /// Allow prints to valgrind log ending with a newline
        ///
        /// See also [`crate::valgrind_printf`]
        #[macro_export]
        macro_rules! valgrind_println {
            () => { $crate::valgrind_printf!("\n") };
            ($($arg:tt)*) => {{
                match std::ffi::CString::from_vec_with_nul(
                    format!("{}\n\0", format_args!($($arg)*)).into_bytes()
                ) {
                    Ok(c_string) => {
                        unsafe {
                            $crate::client_requests::__valgrind_print(
                                c_string.as_ptr() as *const ()
                            );
                        }
                        Ok(())
                    },
                    Err(error) => Err(
                        $crate::client_requests::error::ClientRequestError::from(error)
                    )
                }
            }};
        }

        /// Allow prints to valgrind log ending with a newline
        ///
        /// See also [`crate::valgrind_printf_unchecked`]
        #[macro_export]
        macro_rules! valgrind_println_unchecked {
            () => { $crate::valgrind_printf_unchecked!("\n") };
            ($($arg:tt)*) => {{
                let string = format!("{}\n\0", format_args!($($arg)*));
                $crate::client_requests::__valgrind_print(string.as_ptr() as *const ());
            }};
        }

        /// Allow prints to valgrind log with a backtrace
        ///
        /// See also [`crate::valgrind_printf`]
        #[macro_export]
        macro_rules! valgrind_printf_backtrace {
            ($($arg:tt)*) => {{
                match std::ffi::CString::from_vec_with_nul(
                    format!("{}\0", format_args!($($arg)*)).into_bytes()
                ) {
                    Ok(c_string) => {
                        unsafe {
                            $crate::client_requests::__valgrind_print_backtrace(
                                c_string.as_ptr() as *const ()
                            );
                        }
                        Ok(())
                    },
                    Err(error) => Err(
                        $crate::client_requests::error::ClientRequestError::from(error)
                    )
                }
            }};
        }

        /// Allow prints to valgrind log with a backtrace
        ///
        /// See also [`crate::valgrind_printf_unchecked`]
        #[macro_export]
        macro_rules! valgrind_printf_backtrace_unchecked {
            ($($arg:tt)*) => {{
                let string = format!("{}\0", format_args!($($arg)*));
                $crate::client_requests::__valgrind_print_backtrace(string.as_ptr() as *const ());
            }};
        }

        /// Allow prints to valgrind log with a backtrace ending the formatted string with a newline
        ///
        /// See also [`crate::valgrind_printf`]
        #[macro_export]
        macro_rules! valgrind_println_backtrace {
            () => { $crate::valgrind_printf_backtrace!("\n") };
            ($($arg:tt)*) => {{
                match std::ffi::CString::from_vec_with_nul(
                    format!("{}\n\0", format_args!($($arg)*)).into_bytes()
                ) {
                    Ok(c_string) => {
                        unsafe {
                            $crate::client_requests::__valgrind_print_backtrace(
                                c_string.as_ptr() as *const ()
                            );
                        }
                        Ok(())
                    },
                    Err(error) => Err(
                        $crate::client_requests::error::ClientRequestError::from(error)
                    )
                }
            }};
        }

        /// Allow prints to valgrind log with a backtrace ending the formatted string with a newline
        ///
        /// See also [`crate::valgrind_printf_unchecked`]
        #[macro_export]
        macro_rules! valgrind_println_backtrace_unchecked {
            () => { $crate::valgrind_printf_backtrace_unchecked!("\n") };
            ($($arg:tt)*) => {{
                let string = format!("{}\n\0", format_args!($($arg)*));
                unsafe {
                    $crate::client_requests::__valgrind_print_backtrace(
                        string.as_ptr() as *const ()
                    );
                }
            }};
        }
    } else {
        #[macro_export]
        macro_rules! valgrind_printf {
            ($($arg:tt)*) => { Ok(()) };
        }

        #[macro_export]
        macro_rules! valgrind_printf_unchecked {
            ($($arg:tt)*) => {};
        }

        #[macro_export]
        macro_rules! valgrind_println {
            ($($arg:tt)*) => { Ok(()) };
        }

        #[macro_export]
        macro_rules! valgrind_println_unchecked {
            ($($arg:tt)*) => {};
        }

        #[macro_export]
        macro_rules! valgrind_printf_backtrace {
            ($($arg:tt)*) => { Ok(()) };
        }

        #[macro_export]
        macro_rules! valgrind_printf_backtrace_unchecked {
            ($($arg:tt)*) => {};
        }

        #[macro_export]
        macro_rules! valgrind_println_backtrace {
            ($($arg:tt)*) => { Ok(()) };
        }

        #[macro_export]
        macro_rules! valgrind_println_backtrace_unchecked {
            ($($arg:tt)*) => {};
        }
    }
}

mod arch;
mod bindings;
pub mod callgrind;
pub mod error;
pub mod memcheck;
#[cfg(client_requests_support = "native")]
mod native_bindings;
pub mod valgrind;

use arch::imp::valgrind_do_client_request_expr;
use arch::valgrind_do_client_request_stmt;
use cfg_if::cfg_if;

/// The `ThreadId` is used by some client requests to represent the `tid` which valgrind uses or
/// returns
///
/// This type has no relationship to [`std::thread::ThreadId`]!
pub type ThreadId = usize;

/// The `StackId` is used and returned by some client requests and represents an id on valgrind's
/// stack
pub type StackId = usize;

/// The raw file descriptor number
///
/// This type has no relationship to the standard library type definition of `RawFd` besides they
/// are wrapping the same type on unix systems.
pub type RawFd = cty::c_int;

/// Valgrind's version number from the `valgrind.h` file
///
/// Note that the version numbers were introduced at valgrind version 3.6 and so would not exist in
/// version 3.5 or earlier. `VALGRIND_VERSION` is None is this case, else it is a tuple `(MAJOR,
/// MINOR)`
pub const VALGRIND_VERSION: Option<(u32, u32)> = {
    if bindings::IC_VALGRIND_MAJOR == 0 {
        None
    } else {
        Some((bindings::IC_VALGRIND_MAJOR, bindings::IC_VALGRIND_MINOR))
    }
};

fn fatal_error(func: &str) -> ! {
    panic!(
        "{0}: FATAL: {0}::{func} not available! You may need update your installed valgrind \
         version or don't use this client request. The valgrind version of the valgrind.h header \
         file is {1}. Aborting...",
        module_path!(),
        if let Some((major, minor)) = VALGRIND_VERSION {
            format!("{major}.{minor}")
        } else {
            "< 3.6".to_owned()
        }
    );
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn __valgrind_print(ptr: *const ()) {
    valgrind_do_client_request_expr(
        0,
        bindings::IC_ValgrindClientRequest::IC_PRINTF_VALIST_BY_REF as cty::c_uint,
        ptr as usize,
        0,
        0,
        0,
        0,
    );
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn __valgrind_print_backtrace(ptr: *const ()) {
    valgrind_do_client_request_expr(
        0,
        bindings::IC_ValgrindClientRequest::IC_PRINTF_BACKTRACE_VALIST_BY_REF as cty::c_uint,
        ptr as usize,
        0,
        0,
        0,
        0,
    );
}
