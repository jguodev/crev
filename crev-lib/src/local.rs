use app_dirs::{app_root, AppDataType};
use crev_data::{id::OwnId, level, proof, trust};
use id::{self, LockedId};
use serde_yaml;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use trustdb;
use util::{self, APP_INFO};
use Result;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserConfig {
    pub version: u64,
    #[serde(rename = "current-id")]
    pub current_id: String,
}

/// Local config stored in `~/.config/crev`
pub struct Local {
    root_path: PathBuf,
}

impl Local {
    fn new() -> Result<Self> {
        let root_path = app_root(AppDataType::UserConfig, &APP_INFO)?;
        Ok(Self { root_path })
    }
    pub fn auto_open() -> Result<Self> {
        let repo = Self::new()?;
        if !repo.root_path.exists() {
            bail!("User config not-initialized. Use `crev id gen` to generate CrevID.");
        }

        if !repo.user_config_path().exists() {
            bail!("User config not-initialized. Use `crev id gen` to generate CrevID.");
        }

        Ok(repo)
    }

    pub fn auto_create() -> Result<Self> {
        let repo = Self::new()?;
        fs::create_dir_all(&repo.root_path)?;

        let config_path = repo.user_config_path();
        if config_path.exists() {
            bail!("User config already exists");
        }
        let config: UserConfig = Default::default();
        repo.store_user_config(&config)?;
        Ok(repo)
    }

    pub fn read_current_id(&self) -> Result<String> {
        Ok(self.load_user_config()?.current_id)
    }

    pub fn save_current_id(&self, id: &OwnId) -> Result<()> {
        let mut config = self.load_user_config()?;
        config.current_id = id.pub_key_as_base64();
        self.store_user_config(&config)?;

        Ok(())
    }

    pub fn user_dir_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    fn id_path(&self, id_str: &str) -> PathBuf {
        self.user_dir_path()
            .join("ids")
            .join(format!("{}.yaml", id_str))
    }

    fn user_config_path(&self) -> PathBuf {
        self.user_dir_path().join("config.yaml")
    }

    pub fn load_user_config(&self) -> Result<UserConfig> {
        let path = self.user_config_path();

        let config_str = util::read_file_to_string(&path)?;

        Ok(serde_yaml::from_str(&config_str)?)
    }

    pub fn store_user_config(&self, config: &UserConfig) -> Result<()> {
        let path = self.user_config_path();

        let config_str = serde_yaml::to_string(&config)?;

        util::store_str_to_file(&path, &config_str)
    }

    pub fn read_locked_id(&self) -> Result<LockedId> {
        let config = self.load_user_config()?;
        let path = self.id_path(&config.current_id);
        LockedId::read_from_yaml_file(&path)
    }

    pub fn read_unlocked_id(&self, passphrase: &str) -> Result<OwnId> {
        let locked = self.read_locked_id()?;

        locked.to_unlocked(passphrase)
    }

    pub fn save_locked_id(&self, id: &id::LockedId) -> Result<()> {
        let path = self.id_path(&id.pub_key_as_base64());
        fs::create_dir_all(&path.parent().expect("Not /"))?;
        id.save_to(&path)
    }

    /*
    fn trust_proof_dir_path(&self) -> PathBuf {
        self.user_dir_path().join("trust")
    }

    fn trust_proof_dir_path_for_id(&self, pub_id: &id::PubId) -> PathBuf {
        let id_str = pub_id.id_as_base64();
        self.trust_proof_dir_path().join(id_str)
    }

    fn review_proof_dir_path(&self) -> PathBuf {
        self.user_dir_path().join("review")
    }

    fn review_proof_dir_path_for_id(&self, pub_id: &id::PubId) -> PathBuf {
        let id_str = pub_id.id_as_base64();
        self.review_proof_dir_path().join(id_str)
    }

    pub fn load_all_trust_proof_from(&self, pub_id: &id::PubId) -> Result<Vec<TrustProof>> {
        let path = self.trust_proof_dir_path_for_id(pub_id);
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = util::read_file_to_string(&path)?;

        TrustProof::parse(&content)
    }

    pub fn store_all_trust_proof_from(
        &self,
        pub_id: &id::PubId,
        proofs: &[TrustProof],
    ) -> Result<()> {
        util::store_to_file_with(&self.trust_proof_dir_path_for_id(pub_id), |w| {
            for proof in proofs {
                w.write_all(proof.to_string().as_bytes())?;
            }
            Ok(())
        })
    }


    pub fn load_all_review_proof_from(&self, pub_id: &id::PubId) -> Result<Vec<ReviewProof>> {
        let path = &self.review_proof_dir_path_for_id(pub_id);
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = util::read_file_to_string(&path)?;

        ReviewProof::parse(&content)
    }

    pub fn store_all_review_proof_from(
        &self,
        pub_id: &id::PubId,
        proofs: &[ReviewProof],
    ) -> Result<()> {
        util::store_to_file_with(&self.review_proof_dir_path_for_id(pub_id), |w| {
            for proof in proofs {
                w.write_all(proof.to_string().as_bytes())?;
            }
            Ok(())
        })
    }

    pub fn add_trust_proof_from(&self, pub_id: &id::PubId, proof: TrustProof) -> Result<()> {
        let mut proofs = self.load_all_trust_proof_from(pub_id)?;
        proofs.push(proof);
        self.store_all_trust_proof_from(pub_id, &proofs)?;

        Ok(())
    }

    pub fn add_review_proof_from(&self, pub_id: &id::PubId, proof: ReviewProof) -> Result<()> {
        let mut proofs = self.load_all_review_proof_from(pub_id)?;
        proofs.push(proof);
        self.store_all_review_proof_from(pub_id, &proofs)?;

        Ok(())
    }

    */

    fn get_proof_rel_store_path(&self, proof: &proof::Proof) -> PathBuf {
        PathBuf::from("proofs").join(::proof::rel_store_path(&proof.content))
    }

    fn get_proofs_dir_path(&self) -> PathBuf {
        self.root_path.join("proofs")
    }

    pub fn trust_ids(&self, pub_ids: Vec<String>, passphrase: String) -> Result<()> {
        if pub_ids.is_empty() {
            bail!("No ids to trust. Use `add` first.");
        }
        let id = self.read_unlocked_id(&passphrase)?;

        let trust = trust::TrustBuilder::default()
            .from(id.pub_key_as_base64())
            .from_url(id.id.url.to_owned())
            .from_type(id.id.type_.to_string())
            .comment(Some("".into()))
            .trust(level::Level::Medium)
            .trusted_ids(pub_ids)
            .build()
            .map_err(|e| format_err!("{}", e))?;

        let trust = util::edit_proof_content_iteractively(&trust.into(), proof::ProofType::Trust)?;

        let proof = trust.sign(&id)?;
        let rel_store_path = self.get_proof_rel_store_path(&proof);

        self.append_proof_at(&proof, &rel_store_path)?;
        println!("{}", proof);
        eprintln!("Proof added to your store");
        Ok(())
    }

    pub fn trust_update(&self) -> Result<()> {
        let mut db = trustdb::TrustDB::new();
        let user_config = self.load_user_config()?;
        db.import_recursively(&self.get_proofs_dir_path())?;
        let params = trustdb::TrustDistanceParams {
            max_distance: 10,
            high_trust_distance: 0,
            medium_trust_distance: 1,
            low_trust_distance: 5,
        };

        let trust_set = db.calculate_trust_set(user_config.current_id, &params);

        for id in &trust_set {
            println!("{}", id);
        }
        unimplemented!();
    }

    pub fn append_proof(&self, proof: &proof::Proof) -> Result<()> {
        let rel_store_path = self.get_proof_rel_store_path(proof);
        self.append_proof_at(&proof, &rel_store_path)
    }

    fn append_proof_at(&self, proof: &proof::Proof, rel_store_path: &Path) -> Result<()> {
        let path = self.user_dir_path().join(rel_store_path);

        fs::create_dir_all(path.parent().expect("Not a root dir"))?;
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .write(true)
            .open(path)?;

        file.write_all(proof.to_string().as_bytes())?;
        file.flush()?;

        Ok(())
    }
}