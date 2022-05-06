use anyhow::{bail, Result};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    metadata::Metadata,
    signature::{Keypair, PublicKey, Signature, SignatureSerializer},
};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(bound(archive = "
    <GuaranteeSigned<T> as Archive>::Archived: ::core::fmt::Debug + PartialEq,
",))]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GuarantorSigned<T>
where
    T: Archive,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    pub guarantor: Identity,
    pub data: GuaranteeSigned<T>,
}

impl<T> ::core::ops::Deref for GuarantorSigned<T>
where
    T: Archive,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    type Target = GuaranteeSigned<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Signer<GuaranteeSigned<T>> for GuarantorSigned<T>
where
    T: ::core::fmt::Debug + PartialEq + Archive + Serialize<SignatureSerializer>,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn sign(account: &Account, data: GuaranteeSigned<T>) -> Result<Self>
    where
        Self: Sized,
    {
        if account.account_ref() != data.guarantor {
            bail!("guarantor mismatching");
        }

        Ok(GuarantorSigned {
            guarantor: account.sign(&data)?,
            data,
        })
    }
}

impl<T> Verifier for GuarantorSigned<T>
where
    T: Archive + Serialize<SignatureSerializer> + ::core::fmt::Debug + PartialEq,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        if self.guarantor.account != self.data.data.guarantor {
            bail!("guarantor mismatching");
        }

        self.guarantor.verify(&self.data)?;
        self.data.verify(guarantor)
    }
}

impl<T> Verifier for ArchivedGuarantorSigned<T>
where
    T: Archive + Serialize<SignatureSerializer> + ::core::fmt::Debug + PartialEq,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        if self.guarantor.account != self.data.data.guarantor {
            bail!("guarantor mismatching");
        }

        self.guarantor.verify(&self.data)?;
        self.data.verify(guarantor)
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(bound(archive = "
    <Metadata<T> as Archive>::Archived: ::core::fmt::Debug + PartialEq,
",))]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GuaranteeSigned<T> {
    pub guarantee: Identity,
    pub data: Metadata<T>,
}

impl<T> ::core::ops::Deref for GuaranteeSigned<T>
where
    T: Archive,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    type Target = Metadata<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Signer<Metadata<T>> for GuaranteeSigned<T>
where
    T: Archive + Serialize<SignatureSerializer>,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn sign(account: &Account, data: Metadata<T>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            guarantee: account.sign(&data)?,
            data,
        })
    }
}

impl<T> Verifier for GuaranteeSigned<T>
where
    T: Archive + Serialize<SignatureSerializer>,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        if let Some(guarantor) = guarantor {
            if self.data.guarantor != guarantor {
                bail!("guarantor mismatching");
            }
        }

        self.guarantee.verify(&self.data)
    }
}

impl<T> Verifier for ArchivedGuaranteeSigned<T>
where
    T: Archive + Serialize<SignatureSerializer> + ::core::fmt::Debug + PartialEq,
    <T as Archive>::Archived: ::core::fmt::Debug + PartialEq,
    <Metadata<T> as Archive>::Archived: ::core::fmt::Debug + PartialEq,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        if let Some(guarantor) = guarantor {
            if self.data.guarantor != guarantor {
                bail!("guarantor mismatching");
            }
        }

        self.guarantee.verify(&self.data)
    }
}

pub trait Signer<T>
where
    T: Serialize<SignatureSerializer>,
{
    fn sign(account: &Account, data: T) -> Result<Self>
    where
        Self: Sized;
}

pub trait Verifier {
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()>;
}

impl<T> Verifier for &T
where
    T: Verifier,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        (&**self).verify(guarantor)
    }
}

impl<T> Verifier for Box<T>
where
    T: Verifier,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        (&**self).verify(guarantor)
    }
}

impl<T> Verifier for ::core::pin::Pin<T>
where
    T: ::core::ops::Deref,
    <T as ::core::ops::Deref>::Target: Verifier,
{
    fn verify(&self, guarantor: Option<AccountRef>) -> Result<()> {
        (&**self).verify(guarantor)
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
pub struct Identity {
    pub account: AccountRef,
    pub signature: Signature,
}

impl Identity {
    fn verify<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize<SignatureSerializer>,
    {
        let data = ::rkyv::to_bytes::<_, 64>(data)?;
        self.verify_archived(&data)
    }

    fn verify_archived(&self, data: &[u8]) -> Result<()> {
        use ed25519_dalek::Verifier;

        self.account.public_key.verify(data, &self.signature)?;
        Ok(())
    }
}

impl ArchivedIdentity {
    fn verify<T>(&self, data: &T) -> Result<()> {
        // TODO: to be implemented
        Ok(())
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
pub struct AccountRef {
    pub public_key: PublicKey,
}

impl ::core::ops::Deref for AccountRef {
    type Target = PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.public_key
    }
}

impl ::core::str::FromStr for AccountRef {
    type Err = ::anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            public_key: s.parse()?,
        })
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
pub struct Account {
    pub keypair: Keypair,
}

impl ::core::ops::Deref for Account {
    type Target = Keypair;

    fn deref(&self) -> &Self::Target {
        &self.keypair
    }
}

impl ::core::str::FromStr for Account {
    type Err = ::anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            keypair: s.parse()?,
        })
    }
}

impl Account {
    pub fn generate() -> Self {
        Self {
            keypair: Keypair::generate(),
        }
    }

    pub fn account_ref(&self) -> AccountRef {
        AccountRef {
            public_key: self.keypair.public_key(),
        }
    }

    pub(crate) fn sign<T>(&self, data: &T) -> Result<Identity>
    where
        T: Serialize<SignatureSerializer>,
    {
        use ed25519_dalek::Signer;

        Ok(Identity {
            account: AccountRef {
                public_key: PublicKey(self.keypair.public),
            },
            signature: Signature(self.keypair.sign(&::rkyv::to_bytes(data)?)),
        })
    }
}
