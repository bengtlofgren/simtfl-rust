use std::collections::HashSet;

/// Calculate the notarization threshold used in most permissioned BFT protocols:
/// ceiling(n * 2/3)
#[allow(dead_code)]
fn two_thirds_threshold(n: i32) -> i32 {
    (n * 2 + 2) / 3
}

#[allow(dead_code)]
/// Base trait for BFT blocks and proposals
trait PermissionedBFTBase: std::fmt::Debug {
    fn n(&self) -> i32;
    fn t(&self) -> i32;
    fn parent(&self) -> Option<&dyn PermissionedBFTBase>;
    fn last_final(&self) -> &dyn PermissionedBFTBase;
}

#[allow(dead_code)]
/// Genesis block implementation
#[derive(Debug)]
struct Genesis {
    n: i32,
    t: i32,
}

#[allow(dead_code)]
impl Genesis {
    fn new(n: i32, t: i32) -> Self {
        Genesis { n, t }
    }
}

impl PermissionedBFTBase for Genesis {
    fn n(&self) -> i32 {
        self.n
    }

    fn t(&self) -> i32 {
        self.t
    }

    fn parent(&self) -> Option<&dyn PermissionedBFTBase> {
        None
    }

    fn last_final(&self) -> &dyn PermissionedBFTBase {
        self
    }
}

#[allow(dead_code)]
/// A proposal for a BFT protocol
#[derive(Debug, Clone)]
struct PermissionedBFTProposal<'a> {
    n: i32,
    t: i32,
    parent: &'a dyn PermissionedBFTBase,
    signers: HashSet<i32>,
}

#[allow(dead_code)]
impl<'a> PermissionedBFTProposal<'a> {
    fn new(parent: &'a dyn PermissionedBFTBase) -> Self {
        PermissionedBFTProposal {
            n: parent.n(),
            t: parent.t(),
            parent,
            signers: HashSet::new(),
        }
    }

    fn assert_valid(&self) -> Result<(), &'static str> {
        // TODO: Implement this function
        if self.signers.len() > self.n as usize {
            return Err("Too many signatures");
        }
        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.assert_valid().is_ok()
    }

    fn assert_notarized(&self) -> Result<(), &'static str> {
        self.assert_valid()?;
        if self.signers.len() < self.t as usize {
            return Err("Not enough signatures");
        }
        Ok(())
    }

    fn is_notarized(&self) -> bool {
        self.assert_notarized().is_ok()
    }

    fn add_signature(&mut self, index: i32) -> Result<(), &'static str> {
        self.signers.insert(index);
        if self.signers.len() as i32 > self.n {
            return Err("Too many signatures");
        }
        Ok(())
    }
}

impl<'a> PermissionedBFTBase for PermissionedBFTProposal<'a> {
    fn n(&self) -> i32 {
        self.n
    }

    fn t(&self) -> i32 {
        self.t
    }

    fn parent(&self) -> Option<&dyn PermissionedBFTBase> {
        Some(self.parent)
    }

    fn last_final(&self) -> &dyn PermissionedBFTBase {
        self.parent.last_final()
    }
}

#[allow(dead_code)]
/// A block for a BFT protocol
#[derive(Debug, Clone)]
struct PermissionedBFTBlock<'a> {
    n: i32,
    t: i32,
    proposal: PermissionedBFTProposal<'a>,
}

#[allow(dead_code)]
impl<'a> PermissionedBFTBlock<'a> {
    fn new(proposal: PermissionedBFTProposal<'a>) -> Result<Self, &'static str> {
        proposal.assert_notarized()?;
        Ok(PermissionedBFTBlock {
            n: proposal.n(),
            t: proposal.t(),
            proposal,
        })
    }
}

impl<'a> PermissionedBFTBase for PermissionedBFTBlock<'a> {
    fn n(&self) -> i32 {
        self.n
    }

    fn t(&self) -> i32 {
        self.t
    }

    fn parent(&self) -> Option<&dyn PermissionedBFTBase> {
        Some(self.proposal.parent)
    }

    fn last_final(&self) -> &dyn PermissionedBFTBase {
        if self.parent().is_none() {
            self
        } else {
            self.parent().unwrap().last_final()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // Construct the genesis block
        let genesis = Genesis::new(5, 2);
        let current: &dyn PermissionedBFTBase = &genesis;
        assert_eq!(current.last_final().n(), genesis.n());

        for _ in 0..2 {
            let mut proposal = PermissionedBFTProposal::new(current);
            assert!(proposal.is_valid());
            assert!(!proposal.is_notarized());

            // Not enough signatures
            proposal.add_signature(0).unwrap();
            assert!(!proposal.is_notarized());

            // Same index, so we still only have one signature
            proposal.add_signature(0).unwrap();
            assert!(!proposal.is_notarized());

            // Different index, now we have two signatures as required
            proposal.add_signature(1).unwrap();
            assert!(proposal.is_notarized());

            let block = PermissionedBFTBlock::new(proposal).unwrap();
            assert_eq!(block.last_final().n(), genesis.n());
        }
    }

    #[test]
    fn test_assertions() {
        let genesis = Genesis::new(5, 2);
        let mut proposal = PermissionedBFTProposal::new(&genesis);
        assert!(PermissionedBFTBlock::new(proposal.clone()).is_err());

        proposal.add_signature(0).unwrap();
        assert!(PermissionedBFTBlock::new(proposal.clone()).is_err());

        proposal.add_signature(1).unwrap();
        assert!(PermissionedBFTBlock::new(proposal).is_ok());
    }
}
