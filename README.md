# Vinilo Store — Solana Program (Rust / Anchor)

Smart Contract en Solana que permite gestionar una tienda de discos de vinilo on-chain. Cada propietario tiene una sola cuenta PDA con todo su catálogo almacenado como `Vec<Vinilo>`.

Desarrollado con **Rust** y el framework **Anchor**.

---

## ¿Qué hace el proyecto?

Vinilo Store permite a un propietario crear su tienda en la blockchain y gestionar su catálogo de discos con operaciones CRUD completas. Toda la información vive en una sola cuenta PDA derivada del wallet del propietario.

---

## Instrucciones del Programa

| Instrucción        | Operación  | Descripción                                             |
| ------------------ | ---------- | ------------------------------------------------------- |
| `inaugurarTienda`  | **CREATE** | Crea la PDA de la tienda con catálogo vacío             |
| `agregarVinilo`    | **CREATE** | Añade un disco al `Vec<Vinilo>` de la tienda            |
| `actualizarVinilo` | **UPDATE** | Modifica precio, stock y estado de un disco por álbum   |
| `retirarVinilo`    | **DELETE** | Elimina un disco del catálogo por nombre de álbum       |

---

## Estructura de Datos

### `Tienda` (cuenta PDA — una por propietario)

```rust
pub struct Tienda {
    pub propietario: Pubkey,      // Wallet del dueño
    pub nombre:      String,      // Nombre de la tienda (máx. 50 chars)
    pub catalogo:    Vec<Vinilo>, // Catálogo de discos (máx. 20 vinilos)
}
```

**PDA derivada con:** `["tienda", propietario_wallet]`

---

### `Vinilo` (struct interno — no es una cuenta separada)

```rust
pub struct Vinilo {
    pub artista: String,  // "Pink Floyd"
    pub album:   String,  // "The Dark Side of the Moon"  ← clave de búsqueda
    pub genero:  String,  // "Rock Progresivo"
    pub precio:  u64,     // En centavos MXN (85000 = $850.00)
    pub stock:   u8,      // Unidades disponibles
    pub estado:  String,  // "Disponible" | "Agotado" | "Reservado"
}
```

> **Nota sobre precio:** se almacena en centavos para evitar decimales (tipo `f64` no está disponible en Solana). En el cliente TypeScript se divide entre 100 para mostrar el valor real.

---

## Seguridad

- `has_one = propietario` garantiza que solo el dueño original puede modificar su tienda.
- Se valida on-chain que el estado sea únicamente `"Disponible"`, `"Agotado"` o `"Reservado"`.
- Se previenen duplicados: no puedes registrar el mismo artista + álbum dos veces.
- El catálogo tiene un límite de **20 vinilos**, definido por `InitSpace` al inaugurar.

---

## Cómo usarlo en Solana Playground

### 1. Importar el proyecto

Abre [Solana Playground](https://beta.solpg.io/), pega el enlace de tu repositorio y haz clic en **Import**.

### 2. Conectar Wallet

Haz clic en **Not Connected** (esquina inferior izquierda) → **Continue**. Esto crea tu wallet en Devnet.

### 3. Build & Deploy

1. Clic en **Build** — espera confirmación verde.
2. Copia el Program ID generado y pégalo en `declare_id!("")` en `lib.rs`.
3. Haz **Build** de nuevo y luego **Deploy**.

### 4. Ejecutar el cliente

En la terminal de Solana Playground:

```bash
run
```

Salida esperada:

```
Iniciando Vinilo Store en Solana Devnet...

PDA de la Tienda: 3fyKakf122xDLwSGoRUXCNvsVPVZs4XMV88ZJMEGE3Pr

─────────────────────────────────────────
PASO 1 — Inaugurar la Tienda
─────────────────────────────────────────
Tienda inaugurada con éxito.
   "Disco Vintage MX"
   Vinilos en catálogo: 0

─────────────────────────────────────────
PASO 2 — Agregar Vinilos al Catálogo
─────────────────────────────────────────
'The Dark Side of the Moon' agregado.
'Kind of Blue' agregado.
'IV' agregado.

   Catálogo actual (3 vinilos):
   'The Dark Side of the Moon' — Pink Floyd | Rock Progresivo | $850.00 MXN | Stock: 3 | Disponible
   'Kind of Blue' — Miles Davis | Jazz | $720.00 MXN | Stock: 5 | Disponible
   'IV' — Led Zeppelin | Rock Clásico | $680.00 MXN | Stock: 2 | Disponible

─────────────────────────────────────────
PASO 3 — Actualizar 'Kind of Blue'
─────────────────────────────────────────
Vinilo actualizado.
   Nuevo precio: $650.00 MXN
   Nuevo stock:  0
   Nuevo estado: Agotado

─────────────────────────────────────────
PASO 4 — Retirar 'IV' de Led Zeppelin
─────────────────────────────────────────
Vinilo retirado del catálogo.
   Vinilos restantes: 2

Ciclo CRUD del Vinilo Store completado.
```

---

## Estructura del Proyecto

```
vinilo_store/
├── src/
│   └── lib.rs              # Smart Contract (Rust + Anchor)
├── client/
│   └── client.ts           # Script CRUD interactivo
├── tests/
│   └── anchor.test.ts      # Suite de pruebas
└── README.md
```

---

## Tecnologías

| Herramienta       | Uso                                 |
| ----------------- | ----------------------------------- |
| Rust              | Lógica del Smart Contract           |
| Anchor            | Framework para Solana               |
| TypeScript        | Cliente de pruebas                  |
| Solana Devnet     | Red de pruebas                      |
| Solana Playground | IDE en el navegador                 |

---

## Datos del Proyecto

- **Desarrollador:** Said Sebastian Reyes López
- **Carrera:** TSU en Desarrollo de Software Multiplataforma
- **Program ID:** *(se genera al hacer build en Solana Playground)*
