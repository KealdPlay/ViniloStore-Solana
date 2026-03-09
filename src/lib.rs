use anchor_lang::prelude::*;

// El Program ID se genera automáticamente al hacer "Build" en Solana Playground.
declare_id!("");

// =============================================================================
//  VINILO STORE - Smart Contract en Solana
// =============================================================================
// Programa que permite a un propietario gestionar su tienda de discos de vinilo
// directamente en la blockchain de Solana.
//
// ARQUITECTURA:
//   Una sola cuenta PDA por propietario (semillas: ["tienda", propietario]).
//   Esa cuenta almacena un Vec<Vinilo> con todos los discos del catálogo.
//   Cada vinilo tiene: artista, álbum, género, precio, stock y estado.
//
// INSTRUCCIONES (CRUD):
//   inaugurar_tienda  -> CREATE  - Abre la tienda y reserva espacio en la blockchain
//   agregar_vinilo    -> CREATE  - Añade un disco al catálogo
//   ver_catalogo      -> READ    - Imprime el catálogo completo en el log de la transacción
//   actualizar_vinilo -> UPDATE  - Modifica precio, stock y/o estado de un disco
//   retirar_vinilo    -> DELETE  - Elimina un disco del catálogo por nombre de álbum
//
// SEGURIDAD:
//   has_one = propietario garantiza que solo el dueño original puede
//   modificar su tienda. Anchor valida esto en cada instrucción de escritura.
// =============================================================================

#[program]
pub mod vinilo_store {
    use super::*;

    // =========================================================================
    // CREATE - Inaugurar la Tienda
    // =========================================================================
    // Inicializa la cuenta PDA del propietario. Solo se puede llamar una vez
    // por wallet. Si se llama de nuevo, Anchor arrojará un error porque la
    // cuenta ya existe (init no permite reinicialización).
    //
    // Parámetros:
    //   nombre_tienda -> Nombre de tu tienda
    // =========================================================================
    pub fn inaugurar_tienda(ctx: Context<InaugurarTienda>, nombre_tienda: String) -> Result<()> {
        let tienda = &mut ctx.accounts.tienda;

        tienda.propietario = ctx.accounts.propietario.key();
        tienda.nombre = nombre_tienda.clone();
        tienda.catalogo = Vec::new(); // Catálogo vacío al inaugurar

        msg!(
            "Tienda '{}' inaugurada con éxito en la blockchain!",
            nombre_tienda
        );
        Ok(())
    }

    // =========================================================================
    // CREATE (Add) - Agregar un Vinilo al catálogo
    // =========================================================================
    // Añade un nuevo disco al Vec<Vinilo> de la tienda.
    // Falla si el catálogo ya tiene 20 vinilos (límite definido por InitSpace).
    // Falla si ya existe un disco con el mismo álbum + artista.
    //
    // Parámetros:
    //   artista -> Nombre del artista o banda    (ej. "Pink Floyd")
    //   album   -> Título del álbum              (ej. "The Dark Side of the Moon")
    //   genero  -> Género musical                (ej. "Rock Progresivo")
    //   precio  -> Precio en centavos MXN (u64)  (ej. 85000 = $850.00 MXN)
    //   stock   -> Unidades disponibles (u8)      (ej. 3)
    // =========================================================================
    pub fn agregar_vinilo(
        ctx: Context<GestionarTienda>,
        artista: String,
        album: String,
        genero: String,
        precio: u64,
        stock: u8,
    ) -> Result<()> {
        let tienda = &mut ctx.accounts.tienda;

        // Validar que no se exceda el límite del Vec
        require!(tienda.catalogo.len() < 20, Errores::CatalogoLleno);

        // Evitar duplicados: mismo artista + mismo álbum
        let ya_existe = tienda
            .catalogo
            .iter()
            .any(|v| v.artista == artista && v.album == album);
        require!(!ya_existe, Errores::ViniloYaExiste);

        let nuevo_vinilo = Vinilo {
            artista: artista.clone(),
            album: album.clone(),
            genero,
            precio,
            stock,
            estado: String::from("Disponible"), // Estado inicial por defecto
        };

        tienda.catalogo.push(nuevo_vinilo);

        msg!(
            " Vinilo agregado: '{}' de {} | Precio: ${}.{:02} MXN | Stock: {}",
            album,
            artista,
            precio / 100,
            precio % 100,
            stock
        );
        Ok(())
    }

    // =========================================================================
    // READ - Ver el catálogo completo
    // =========================================================================
    // Imprime en el log de la transacción todos los discos del catálogo,
    // incluyendo artista, álbum, género, precio, stock y estado.
    //
    // Nota técnica: en Solana el estado de una cuenta también puede leerse
    // off-chain con program.account.tienda.fetch(pda) sin costo de transacción.
    // Esta instrucción existe para completar el CRUD on-chain y permite
    // verificar el catálogo directamente desde el explorador de Solana.
    //
    // Parámetros: ninguno.
    // =========================================================================
    pub fn ver_catalogo(ctx: Context<GestionarTienda>) -> Result<()> {
        let tienda = &ctx.accounts.tienda;

        msg!(
            "Catalogo de '{}' | Total: {} vinilo(s)",
            tienda.nombre,
            tienda.catalogo.len()
        );

        if tienda.catalogo.is_empty() {
            msg!("   (El catálogo está vacío)");
            return Ok(());
        }

        for (i, v) in tienda.catalogo.iter().enumerate() {
            msg!(
                "   [{}] '{}' - {} | Género: {} | ${}.{:02} MXN | Stock: {} | {}",
                i + 1,
                v.album,
                v.artista,
                v.genero,
                v.precio / 100,
                v.precio % 100,
                v.stock,
                v.estado,
            );
        }

        Ok(())
    }

    // =========================================================================
    // UPDATE - Actualizar datos de un Vinilo
    // =========================================================================
    // Busca un disco por nombre de álbum y actualiza su precio, stock y estado.
    // Solo el propietario puede llamar esta instrucción (validado por has_one).
    //
    // Parámetros:
    //   album        -> Álbum a buscar (clave de búsqueda en el catálogo)
    //   nuevo_precio -> Nuevo precio en centavos MXN
    //   nuevo_stock  -> Nueva cantidad de unidades disponibles
    //   nuevo_estado -> "Disponible", "Agotado" o "Reservado"
    // =========================================================================
    pub fn actualizar_vinilo(
        ctx: Context<GestionarTienda>,
        album: String,
        nuevo_precio: u64,
        nuevo_stock: u8,
        nuevo_estado: String,
    ) -> Result<()> {
        // Validar que el estado sea uno de los tres valores permitidos
        require!(
            nuevo_estado == "Disponible"
                || nuevo_estado == "Agotado"
                || nuevo_estado == "Reservado",
            Errores::EstadoInvalido
        );

        let catalogo = &mut ctx.accounts.tienda.catalogo;

        // Buscar el índice del vinilo por nombre de álbum
        if let Some(pos) = catalogo.iter().position(|v| v.album == album) {
            catalogo[pos].precio = nuevo_precio;
            catalogo[pos].stock = nuevo_stock;
            catalogo[pos].estado = nuevo_estado.clone();

            msg!(
                "  '{}' actualizado -> Precio: ${}.{:02} | Stock: {} | Estado: {}",
                album,
                nuevo_precio / 100,
                nuevo_precio % 100,
                nuevo_stock,
                nuevo_estado
            );
            return Ok(());
        }

        Err(Errores::ViniloNoEncontrado.into())
    }

    // =========================================================================
    // DELETE - Retirar un Vinilo del catálogo
    // =========================================================================
    // Busca un disco por nombre de álbum y lo elimina del Vec.
    // Nota: esto NO libera la cuenta de la blockchain (el espacio sigue reservado).
    // El tamaño de la cuenta fue fijado en la inauguración con InitSpace.
    //
    // Parámetros:
    //   album -> Título del álbum a eliminar
    // =========================================================================
    pub fn retirar_vinilo(ctx: Context<GestionarTienda>, album: String) -> Result<()> {
        let catalogo = &mut ctx.accounts.tienda.catalogo;

        if let Some(pos) = catalogo.iter().position(|v| v.album == album) {
            catalogo.remove(pos);
            msg!("Retirado: Vinilo '{}' retirado del catálogo.", album);
            return Ok(());
        }

        Err(Errores::ViniloNoEncontrado.into())
    }
}

// =============================================================================
// ESTRUCTURAS DE DATOS
// =============================================================================

/// Struct secundario (no es una cuenta). Representa un disco individual.
/// Se serializa/deserializa dentro del Vec<Vinilo> de la cuenta Tienda.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Vinilo {
    #[max_len(60)]
    pub artista: String, // Nombre del artista o banda
    #[max_len(80)]
    pub album: String, // Título del álbum (también sirve como clave de búsqueda)
    #[max_len(40)]
    pub genero: String, // Género musical
    pub precio: u64, // Precio en centavos MXN (ej. 85000 = $850.00)
    pub stock: u8,   // Unidades disponibles
    #[max_len(20)]
    pub estado: String, // "Disponible" | "Agotado" | "Reservado"
}

/// Cuenta principal de la tienda. Una PDA por propietario.
/// Contiene el catálogo completo como Vec<Vinilo> (máximo 20 discos).
#[account]
#[derive(InitSpace)]
pub struct Tienda {
    pub propietario: Pubkey, // 32 bytes - Wallet del dueño de la tienda
    #[max_len(50)]
    pub nombre: String, // Nombre de la tienda
    #[max_len(20)] // Límite de 20 vinilos por tienda
    pub catalogo: Vec<Vinilo>,
}

// =============================================================================
// CONTEXTOS DE INSTRUCCIÓN (Validación de cuentas)
// =============================================================================

/// Contexto para inaugurar la tienda (CREATE).
/// `init` garantiza que la cuenta se crea una sola vez.
/// La PDA se deriva de la semilla "tienda" + la clave pública del propietario,
/// lo que hace que cada wallet tenga exactamente una tienda única.
#[derive(Accounts)]
pub struct InaugurarTienda<'info> {
    #[account(mut)]
    pub propietario: Signer<'info>,

    #[account(
        init,
        payer = propietario,
        space = 8 + Tienda::INIT_SPACE,          // 8 bytes de discriminador + tamaño calculado
        seeds = [b"tienda", propietario.key().as_ref()],
        bump
    )]
    pub tienda: Account<'info, Tienda>,

    pub system_program: Program<'info, System>,
}

/// Contexto compartido para READ, UPDATE y DELETE.
/// Para READ no se necesita `mut` en la cuenta, pero reutilice este contexto
/// por simplicidad - Anchor solo escribirá si la instrucción lo hace explícitamente.
/// `has_one = propietario` valida que quien firma sea el mismo dueño registrado
/// en la cuenta, impidiendo que wallets ajenos accedan a la tienda.
#[derive(Accounts)]
pub struct GestionarTienda<'info> {
    pub propietario: Signer<'info>,

    #[account(
        mut,
        has_one = propietario @ Errores::NoEresElPropietario,
        seeds = [b"tienda", propietario.key().as_ref()],
        bump
    )]
    pub tienda: Account<'info, Tienda>,
}

// =============================================================================
// CÓDIGOS DE ERROR
// =============================================================================
#[error_code]
pub enum Errores {
    #[msg("No tienes permisos: no eres el propietario de esta tienda.")]
    NoEresElPropietario,

    #[msg("El catálogo ya tiene 20 vinilos. Retira uno antes de agregar otro.")]
    CatalogoLleno,

    #[msg("Ese álbum no está en el catálogo.")]
    ViniloNoEncontrado,

    #[msg("Ese vinilo ya existe en el catálogo (mismo artista y álbum).")]
    ViniloYaExiste,

    #[msg("Estado inválido. Usa únicamente: 'Disponible', 'Agotado' o 'Reservado'.")]
    EstadoInvalido,
}
