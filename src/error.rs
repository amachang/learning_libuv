#![allow(non_upper_case_globals)]

use super::{
    native::*,
};

use std::{
    io,
    fmt,
    ffi::CStr,
    fmt::{
        Display,
        Formatter,
    },
    sync::PoisonError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        let kind = ErrorKind::Message(message);
        Error { kind }
    }
}

impl From<uv_errno_t> for Error {
    fn from(native: uv_errno_t) -> Self {
        let kind = NativeErrorKind::from_native(native);
        let kind = if let Some(kind) = kind {
            ErrorKind::NativeError(kind)
        } else {
            ErrorKind::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unknown errno given as uv_errno_t: {}", native),
            ))
        };
        Error { kind }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        let kind = ErrorKind::Message(err.to_string());
        Error { kind }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        let kind = ErrorKind::IoError(err);
        Error { kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UvError({})", self.kind)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NativeError(NativeErrorKind),
    IoError(io::Error),
    Message(String),
    LockPoisonedError(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeError(kind) => write!(f, "Native({})", kind),
            Self::IoError(err) => write!(f, "Io({})", err),
            Self::Message(message) => write!(f, "Message({})", message),
            Self::LockPoisonedError(message) => write!(f, "LockPoisoned({})", message),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NativeErrorKind {
    E2BIG,
    EACCES,
    EADDRINUSE,
    EADDRNOTAVAIL,
    EAFNOSUPPORT,
    EAGAIN,
    EAIADDRFAMILY,
    EAIAGAIN,
    EAIBADFLAGS,
    EAIBADHINTS,
    EAICANCELED,
    EAIFAIL,
    EAIFAMILY,
    EAIMEMORY,
    EAINODATA,
    EAINONAME,
    EAIOVERFLOW,
    EAIPROTOCOL,
    EAISERVICE,
    EAISOCKTYPE,
    EALREADY,
    EBADF,
    EBUSY,
    ECANCELED,
    ECHARSET,
    ECONNABORTED,
    ECONNREFUSED,
    ECONNRESET,
    EDESTADDRREQ,
    EEXIST,
    EFAULT,
    EFBIG,
    EHOSTUNREACH,
    EINTR,
    EINVAL,
    EIO,
    EISCONN,
    EISDIR,
    ELOOP,
    EMFILE,
    EMSGSIZE,
    ENAMETOOLONG,
    ENETDOWN,
    ENETUNREACH,
    ENFILE,
    ENOBUFS,
    ENODEV,
    ENOENT,
    ENOMEM,
    ENONET,
    ENOPROTOOPT,
    ENOSPC,
    ENOSYS,
    ENOTCONN,
    ENOTDIR,
    ENOTEMPTY,
    ENOTSOCK,
    ENOTSUP,
    EOVERFLOW,
    EPERM,
    EPIPE,
    EPROTO,
    EPROTONOSUPPORT,
    EPROTOTYPE,
    ERANGE,
    EROFS,
    ESHUTDOWN,
    ESPIPE,
    ESRCH,
    ETIMEDOUT,
    ETXTBSY,
    EXDEV,
    UNKNOWN,
    EOF,
    ENXIO,
    EMLINK,
    EHOSTDOWN,
    EREMOTEIO,
    ENOTTY,
    EFTYPE,
    EILSEQ,
    ESOCKTNOSUPPORT,
    ENODATA,
}

impl NativeErrorKind {
    pub fn to_native(&self) -> uv_errno_t {
        match self {
            Self::E2BIG => uv_errno_t_UV_E2BIG,
            Self::EACCES => uv_errno_t_UV_EACCES,
            Self::EADDRINUSE => uv_errno_t_UV_EADDRINUSE,
            Self::EADDRNOTAVAIL => uv_errno_t_UV_EADDRNOTAVAIL,
            Self::EAFNOSUPPORT => uv_errno_t_UV_EAFNOSUPPORT,
            Self::EAGAIN => uv_errno_t_UV_EAGAIN,
            Self::EAIADDRFAMILY => uv_errno_t_UV_EAI_ADDRFAMILY,
            Self::EAIAGAIN => uv_errno_t_UV_EAI_AGAIN,
            Self::EAIBADFLAGS => uv_errno_t_UV_EAI_BADFLAGS,
            Self::EAIBADHINTS => uv_errno_t_UV_EAI_BADHINTS,
            Self::EAICANCELED => uv_errno_t_UV_EAI_CANCELED,
            Self::EAIFAIL => uv_errno_t_UV_EAI_FAIL,
            Self::EAIFAMILY => uv_errno_t_UV_EAI_FAMILY,
            Self::EAIMEMORY => uv_errno_t_UV_EAI_MEMORY,
            Self::EAINODATA => uv_errno_t_UV_EAI_NODATA,
            Self::EAINONAME => uv_errno_t_UV_EAI_NONAME,
            Self::EAIOVERFLOW => uv_errno_t_UV_EAI_OVERFLOW,
            Self::EAIPROTOCOL => uv_errno_t_UV_EAI_PROTOCOL,
            Self::EAISERVICE => uv_errno_t_UV_EAI_SERVICE,
            Self::EAISOCKTYPE => uv_errno_t_UV_EAI_SOCKTYPE,
            Self::EALREADY => uv_errno_t_UV_EALREADY,
            Self::EBADF => uv_errno_t_UV_EBADF,
            Self::EBUSY => uv_errno_t_UV_EBUSY,
            Self::ECANCELED => uv_errno_t_UV_ECANCELED,
            Self::ECHARSET => uv_errno_t_UV_ECHARSET,
            Self::ECONNABORTED => uv_errno_t_UV_ECONNABORTED,
            Self::ECONNREFUSED => uv_errno_t_UV_ECONNREFUSED,
            Self::ECONNRESET => uv_errno_t_UV_ECONNRESET,
            Self::EDESTADDRREQ => uv_errno_t_UV_EDESTADDRREQ,
            Self::EEXIST => uv_errno_t_UV_EEXIST,
            Self::EFAULT => uv_errno_t_UV_EFAULT,
            Self::EFBIG => uv_errno_t_UV_EFBIG,
            Self::EHOSTUNREACH => uv_errno_t_UV_EHOSTUNREACH,
            Self::EINTR => uv_errno_t_UV_EINTR,
            Self::EINVAL => uv_errno_t_UV_EINVAL,
            Self::EIO => uv_errno_t_UV_EIO,
            Self::EISCONN => uv_errno_t_UV_EISCONN,
            Self::EISDIR => uv_errno_t_UV_EISDIR,
            Self::ELOOP => uv_errno_t_UV_ELOOP,
            Self::EMFILE => uv_errno_t_UV_EMFILE,
            Self::EMSGSIZE => uv_errno_t_UV_EMSGSIZE,
            Self::ENAMETOOLONG => uv_errno_t_UV_ENAMETOOLONG,
            Self::ENETDOWN => uv_errno_t_UV_ENETDOWN,
            Self::ENETUNREACH => uv_errno_t_UV_ENETUNREACH,
            Self::ENFILE => uv_errno_t_UV_ENFILE,
            Self::ENOBUFS => uv_errno_t_UV_ENOBUFS,
            Self::ENODEV => uv_errno_t_UV_ENODEV,
            Self::ENOENT => uv_errno_t_UV_ENOENT,
            Self::ENOMEM => uv_errno_t_UV_ENOMEM,
            Self::ENONET => uv_errno_t_UV_ENONET,
            Self::ENOPROTOOPT => uv_errno_t_UV_ENOPROTOOPT,
            Self::ENOSPC => uv_errno_t_UV_ENOSPC,
            Self::ENOSYS => uv_errno_t_UV_ENOSYS,
            Self::ENOTCONN => uv_errno_t_UV_ENOTCONN,
            Self::ENOTDIR => uv_errno_t_UV_ENOTDIR,
            Self::ENOTEMPTY => uv_errno_t_UV_ENOTEMPTY,
            Self::ENOTSOCK => uv_errno_t_UV_ENOTSOCK,
            Self::ENOTSUP => uv_errno_t_UV_ENOTSUP,
            Self::EOVERFLOW => uv_errno_t_UV_EOVERFLOW,
            Self::EPERM => uv_errno_t_UV_EPERM,
            Self::EPIPE => uv_errno_t_UV_EPIPE,
            Self::EPROTO => uv_errno_t_UV_EPROTO,
            Self::EPROTONOSUPPORT => uv_errno_t_UV_EPROTONOSUPPORT,
            Self::EPROTOTYPE => uv_errno_t_UV_EPROTOTYPE,
            Self::ERANGE => uv_errno_t_UV_ERANGE,
            Self::EROFS => uv_errno_t_UV_EROFS,
            Self::ESHUTDOWN => uv_errno_t_UV_ESHUTDOWN,
            Self::ESPIPE => uv_errno_t_UV_ESPIPE,
            Self::ESRCH => uv_errno_t_UV_ESRCH,
            Self::ETIMEDOUT => uv_errno_t_UV_ETIMEDOUT,
            Self::ETXTBSY => uv_errno_t_UV_ETXTBSY,
            Self::EXDEV => uv_errno_t_UV_EXDEV,
            Self::UNKNOWN => uv_errno_t_UV_UNKNOWN,
            Self::EOF => uv_errno_t_UV_EOF,
            Self::ENXIO => uv_errno_t_UV_ENXIO,
            Self::EMLINK => uv_errno_t_UV_EMLINK,
            Self::EHOSTDOWN => uv_errno_t_UV_EHOSTDOWN,
            Self::EREMOTEIO => uv_errno_t_UV_EREMOTEIO,
            Self::ENOTTY => uv_errno_t_UV_ENOTTY,
            Self::EFTYPE => uv_errno_t_UV_EFTYPE,
            Self::EILSEQ => uv_errno_t_UV_EILSEQ,
            Self::ESOCKTNOSUPPORT => uv_errno_t_UV_ESOCKTNOSUPPORT,
            Self::ENODATA => uv_errno_t_UV_ENODATA,
        }
    }

    pub fn from_native(native: uv_errno_t) -> Option<Self> {
        Some(match native {
            uv_errno_t_UV_E2BIG => Self::E2BIG,
            uv_errno_t_UV_EACCES => Self::EACCES,
            uv_errno_t_UV_EADDRINUSE => Self::EADDRINUSE,
            uv_errno_t_UV_EADDRNOTAVAIL => Self::EADDRNOTAVAIL,
            uv_errno_t_UV_EAFNOSUPPORT => Self::EAFNOSUPPORT,
            uv_errno_t_UV_EAGAIN => Self::EAGAIN,
            uv_errno_t_UV_EAI_ADDRFAMILY => Self::EAIADDRFAMILY,
            uv_errno_t_UV_EAI_AGAIN => Self::EAIAGAIN,
            uv_errno_t_UV_EAI_BADFLAGS => Self::EAIBADFLAGS,
            uv_errno_t_UV_EAI_BADHINTS => Self::EAIBADHINTS,
            uv_errno_t_UV_EAI_CANCELED => Self::EAICANCELED,
            uv_errno_t_UV_EAI_FAIL => Self::EAIFAIL,
            uv_errno_t_UV_EAI_FAMILY => Self::EAIFAMILY,
            uv_errno_t_UV_EAI_MEMORY => Self::EAIMEMORY,
            uv_errno_t_UV_EAI_NODATA => Self::EAINODATA,
            uv_errno_t_UV_EAI_NONAME => Self::EAINONAME,
            uv_errno_t_UV_EAI_OVERFLOW => Self::EAIOVERFLOW,
            uv_errno_t_UV_EAI_PROTOCOL => Self::EAIPROTOCOL,
            uv_errno_t_UV_EAI_SERVICE => Self::EAISERVICE,
            uv_errno_t_UV_EAI_SOCKTYPE => Self::EAISOCKTYPE,
            uv_errno_t_UV_EALREADY => Self::EALREADY,
            uv_errno_t_UV_EBADF => Self::EBADF,
            uv_errno_t_UV_EBUSY => Self::EBUSY,
            uv_errno_t_UV_ECANCELED => Self::ECANCELED,
            uv_errno_t_UV_ECHARSET => Self::ECHARSET,
            uv_errno_t_UV_ECONNABORTED => Self::ECONNABORTED,
            uv_errno_t_UV_ECONNREFUSED => Self::ECONNREFUSED,
            uv_errno_t_UV_ECONNRESET => Self::ECONNRESET,
            uv_errno_t_UV_EDESTADDRREQ => Self::EDESTADDRREQ,
            uv_errno_t_UV_EEXIST => Self::EEXIST,
            uv_errno_t_UV_EFAULT => Self::EFAULT,
            uv_errno_t_UV_EFBIG => Self::EFBIG,
            uv_errno_t_UV_EHOSTUNREACH => Self::EHOSTUNREACH,
            uv_errno_t_UV_EINTR => Self::EINTR,
            uv_errno_t_UV_EINVAL => Self::EINVAL,
            uv_errno_t_UV_EIO => Self::EIO,
            uv_errno_t_UV_EISCONN => Self::EISCONN,
            uv_errno_t_UV_EISDIR => Self::EISDIR,
            uv_errno_t_UV_ELOOP => Self::ELOOP,
            uv_errno_t_UV_EMFILE => Self::EMFILE,
            uv_errno_t_UV_EMSGSIZE => Self::EMSGSIZE,
            uv_errno_t_UV_ENAMETOOLONG => Self::ENAMETOOLONG,
            uv_errno_t_UV_ENETDOWN => Self::ENETDOWN,
            uv_errno_t_UV_ENETUNREACH => Self::ENETUNREACH,
            uv_errno_t_UV_ENFILE => Self::ENFILE,
            uv_errno_t_UV_ENOBUFS => Self::ENOBUFS,
            uv_errno_t_UV_ENODEV => Self::ENODEV,
            uv_errno_t_UV_ENOENT => Self::ENOENT,
            uv_errno_t_UV_ENOMEM => Self::ENOMEM,
            uv_errno_t_UV_ENONET => Self::ENONET,
            uv_errno_t_UV_ENOPROTOOPT => Self::ENOPROTOOPT,
            uv_errno_t_UV_ENOSPC => Self::ENOSPC,
            uv_errno_t_UV_ENOSYS => Self::ENOSYS,
            uv_errno_t_UV_ENOTCONN => Self::ENOTCONN,
            uv_errno_t_UV_ENOTDIR => Self::ENOTDIR,
            uv_errno_t_UV_ENOTEMPTY => Self::ENOTEMPTY,
            uv_errno_t_UV_ENOTSOCK => Self::ENOTSOCK,
            uv_errno_t_UV_ENOTSUP => Self::ENOTSUP,
            uv_errno_t_UV_EOVERFLOW => Self::EOVERFLOW,
            uv_errno_t_UV_EPERM => Self::EPERM,
            uv_errno_t_UV_EPIPE => Self::EPIPE,
            uv_errno_t_UV_EPROTO => Self::EPROTO,
            uv_errno_t_UV_EPROTONOSUPPORT => Self::EPROTONOSUPPORT,
            uv_errno_t_UV_EPROTOTYPE => Self::EPROTOTYPE,
            uv_errno_t_UV_ERANGE => Self::ERANGE,
            uv_errno_t_UV_EROFS => Self::EROFS,
            uv_errno_t_UV_ESHUTDOWN => Self::ESHUTDOWN,
            uv_errno_t_UV_ESPIPE => Self::ESPIPE,
            uv_errno_t_UV_ESRCH => Self::ESRCH,
            uv_errno_t_UV_ETIMEDOUT => Self::ETIMEDOUT,
            uv_errno_t_UV_ETXTBSY => Self::ETXTBSY,
            uv_errno_t_UV_EXDEV => Self::EXDEV,
            uv_errno_t_UV_UNKNOWN => Self::UNKNOWN,
            uv_errno_t_UV_EOF => Self::EOF,
            uv_errno_t_UV_ENXIO => Self::ENXIO,
            uv_errno_t_UV_EMLINK => Self::EMLINK,
            uv_errno_t_UV_EHOSTDOWN => Self::EHOSTDOWN,
            uv_errno_t_UV_EREMOTEIO => Self::EREMOTEIO,
            uv_errno_t_UV_ENOTTY => Self::ENOTTY,
            uv_errno_t_UV_EFTYPE => Self::EFTYPE,
            uv_errno_t_UV_EILSEQ => Self::EILSEQ,
            uv_errno_t_UV_ESOCKTNOSUPPORT => Self::ESOCKTNOSUPPORT,
            uv_errno_t_UV_ENODATA => Self::ENODATA,
            _ => return None,
        })
    }
}

impl Display for NativeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let err_str = unsafe { CStr::from_ptr(uv_strerror(self.to_native())) };
        write!(f, "{}", err_str.to_string_lossy())
    }
}

