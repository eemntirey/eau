//! `courants` — les canaux typés du cahier §5.
//!
//! Propriétés posées dès la Vague 0, testées sur l'hôte :
//!
//! - **Typés** : un Courant porte un [`Schema`] nommé et versionné ; deux
//!   schémas incompatibles sont refusés à l'ouverture.
//! - **Écluses** : chaque Courant a une capacité fixe ; quand il est plein,
//!   l'émetteur reçoit de la contre-pression ([`Courant::envoyer`] rend
//!   l'erreur avec la valeur), jamais une perte silencieuse ni une mémoire
//!   infinie.
//! - **Observables** : la journalisation est prévue par le type, pas greffée
//!   après coup (Vague 1 pour l'écoute en lecture seule par clé dédiée).
//!
//! Vague 0 : file circulaire mono-producteur mono-consommateur, sans
//! allocation dynamique — le noyau n'a pas encore d'allocateur.

#![no_std]
#![deny(unsafe_code)]

/// Schéma d'un Courant : nom et version, vérifiés à l'ouverture.
///
/// Deux schémas sont compatibles si et seulement si le nom et la version
/// majeure coïncident. Un émetteur et un récepteur qui ne se comprennent
/// pas n'ouvrent jamais le canal (§5 : « un schéma incompatible est refusé »).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schema {
    pub nom: &'static str,
    pub version: u16,
}

impl Schema {
    pub const fn nouveau(nom: &'static str, version: u16) -> Self {
        Self { nom, version }
    }

    /// Compatibilité : même nom, même version majeure.
    pub fn compatible_avec(&self, autre: &Schema) -> bool {
        self.nom == autre.nom && self.version == autre.version
    }
}

/// Erreurs d'un Courant. Aucune perte silencieuse : chaque échec est nommé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Erreur {
    /// Le Courant est plein : l'émetteur garde sa valeur (contre-pression).
    Plein,
    /// Le Courant est vide : le récepteur réessaiera.
    Vide,
    /// Le schéma du pair est incompatible : le canal ne s'ouvre pas.
    SchemaIncompatible,
}

/// Un Courant : file circulaire à capacité fixe, sans allocation.
///
/// `N` est la capacité de l'écluse. La contre-pression est la règle :
/// `envoyer` sur un Courant plein rend la valeur, il ne l'écrase jamais.
pub struct Courant<T, const N: usize> {
    schema: Schema,
    tampon: [Option<T>; N],
    tete: usize,
    longueur: usize,
}

impl<T: Copy, const N: usize> Courant<T, N> {
    /// Ouvre un Courant si les deux schémas sont compatibles, sinon refuse.
    pub fn ouvrir(schema_local: Schema, schema_pair: Schema) -> Result<Self, Erreur> {
        if !schema_local.compatible_avec(&schema_pair) {
            return Err(Erreur::SchemaIncompatible);
        }
        Ok(Self {
            schema: schema_local,
            tampon: [None; N],
            tete: 0,
            longueur: 0,
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn capacite(&self) -> usize {
        N
    }

    pub fn longueur(&self) -> usize {
        self.longueur
    }

    /// Envoie une valeur. Courant plein → `Erreur::Plein` et la valeur
    /// revient à l'émetteur (écluse, §5).
    pub fn envoyer(&mut self, valeur: T) -> Result<(), (T, Erreur)> {
        if self.longueur == N {
            return Err((valeur, Erreur::Plein));
        }
        let queue = (self.tete + self.longueur) % N;
        self.tampon[queue] = Some(valeur);
        self.longueur += 1;
        Ok(())
    }

    /// Reçoit la valeur la plus ancienne. Courant vide → `Erreur::Vide`.
    pub fn recevoir(&mut self) -> Result<T, Erreur> {
        if self.longueur == 0 {
            return Err(Erreur::Vide);
        }
        let valeur = self.tampon[self.tete].take();
        self.tete = (self.tete + 1) % N;
        self.longueur -= 1;
        // Invariant : take() sur une case dans [tete, tete+longueur) échoue
        // seulement si un bug a corrompu la file ; on ne peut pas y arriver
        // si envoyer/recevoir sont les seuls accès au tampon.
        valeur.ok_or(Erreur::Vide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAVIER: Schema = Schema::nouveau("clavier", 1);

    #[test]
    fn ouverture_refuse_schemas_incompatibles() {
        let autre_version = Schema::nouveau("clavier", 2);
        let autre_nom = Schema::nouveau("souris", 1);
        assert!(Courant::<u16, 4>::ouvrir(CLAVIER, autre_version).is_err());
        assert!(Courant::<u16, 4>::ouvrir(CLAVIER, autre_nom).is_err());
        assert!(Courant::<u16, 4>::ouvrir(CLAVIER, CLAVIER).is_ok());
    }

    #[test]
    fn contre_pression_a_la_capacite_pas_de_perte() {
        let mut c: Courant<u16, 2> = Courant::ouvrir(CLAVIER, CLAVIER).unwrap();
        assert_eq!(c.envoyer(10), Ok(()));
        assert_eq!(c.envoyer(20), Ok(()));
        // Écluse pleine : la valeur revient, rien n'est écrasé.
        assert_eq!(c.envoyer(30), Err((30, Erreur::Plein)));
        assert_eq!(c.longueur(), 2);
        // Ordre FIFO préservé.
        assert_eq!(c.recevoir(), Ok(10));
        assert_eq!(c.recevoir(), Ok(20));
        assert_eq!(c.recevoir(), Err(Erreur::Vide));
    }

    #[test]
    fn la_file_boucle_sur_son_anneau() {
        let mut c: Courant<u8, 3> = Courant::ouvrir(CLAVIER, CLAVIER).unwrap();
        // Fenêtre glissante : l'anneau fait plusieurs tours complets,
        // chaque valeur ressort dans l'ordre d'entrée, jamais d'écrasement.
        for v in 0u8..10 {
            assert_eq!(c.envoyer(v), Ok(()));
            if c.longueur() == 3 {
                assert_eq!(c.recevoir(), Ok(v - 2));
            }
        }
        // Il reste les deux dernières valeurs jamais sorties.
        assert_eq!(c.recevoir(), Ok(8));
        assert_eq!(c.recevoir(), Ok(9));
        assert_eq!(c.recevoir(), Err(Erreur::Vide));
    }
}
