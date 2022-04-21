use crate::blockchain::sized_bytes::{Bytes48, Bytes96, SizedBytes};
use blst::min_pk::{PublicKey, Signature};
use blst::BLST_ERROR;

//const BASIC_SCHEME_DST: &[u8; 43] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
const AUG_SCHEME_DST: &[u8; 43] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";
// const POP_SCHEME_DST: &[u8; 43] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
// const AUG_SCHEME_POP_DST: &[u8; 43] = b"BLS_POP_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

pub fn verify_signature(public_key: &Bytes48, msg: &[u8], signature: &Bytes96) -> bool {
    let sig = Signature::from_bytes(signature.to_bytes().as_slice()).unwrap();
    let pk = PublicKey::from_bytes(public_key.to_bytes().as_slice()).unwrap();
    match sig.verify(
        true,
        &msg,
        AUG_SCHEME_DST,
        &public_key.to_bytes().as_slice(),
        &pk,
        true,
    ) {
        BLST_ERROR::BLST_SUCCESS => {
            return true;
        }
        BLST_ERROR::BLST_BAD_ENCODING => {
            return false;
        }
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => {
            return false;
        }
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => {
            return false;
        }
        BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => {
            return false;
        }
        BLST_ERROR::BLST_VERIFY_FAIL => {
            return false;
        }
        BLST_ERROR::BLST_PK_IS_INFINITY => {
            return false;
        }
        BLST_ERROR::BLST_BAD_SCALAR => {
            return false;
        }
    }
}

pub fn aggregate_verify_signature(
    public_keys: &Vec<&Bytes48>,
    msgs: &Vec<&[u8]>,
    signature: &Bytes96,
) -> bool {
    let sig: Signature = Signature::from_bytes(&signature.to_bytes().as_slice()).unwrap();
    let mut new_msgs: Vec<Vec<u8>> = Vec::new();
    let mut keys: Vec<PublicKey> = Vec::new();
    for (key, msg) in public_keys.iter().zip(msgs) {
        let mut combined = Vec::new();
        combined.extend(key.to_bytes().as_slice());
        combined.extend(*msg);
        new_msgs.push(combined);
        let pk = PublicKey::from_bytes(key.to_bytes().as_slice()).unwrap();
        keys.push(pk);
    }
    let _msgs: Vec<&[u8]> = new_msgs.iter().map(|e| e.as_slice()).collect();
    let _keys: Vec<&PublicKey> = keys.iter().map(|e| e).collect();
    match sig.aggregate_verify(
        true,
        _msgs.as_slice(),
        AUG_SCHEME_DST,
        _keys.as_slice(),
        true,
    ) {
        BLST_ERROR::BLST_SUCCESS => {
            return true;
        }
        BLST_ERROR::BLST_BAD_ENCODING => {
            return false;
        }
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => {
            return false;
        }
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => {
            return false;
        }
        BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => {
            return false;
        }
        BLST_ERROR::BLST_VERIFY_FAIL => {
            return false;
        }
        BLST_ERROR::BLST_PK_IS_INFINITY => {
            return false;
        }
        BLST_ERROR::BLST_BAD_SCALAR => {
            return false;
        }
    }
}
