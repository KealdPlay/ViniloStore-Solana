// client/client.ts
// =============================================================================
// VINILO STORE - Script de Cliente (Solana Playground)
// =============================================================================

// Espera a que una TX este finalizada en todos los nodos de la red
async function esperar(tx: string) {
  await pg.connection.confirmTransaction(tx, "finalized");
}

console.log("Iniciando Vinilo Store en Solana Devnet...\n");
console.log("Propietario:", pg.wallet.publicKey.toString());

const balance = await pg.connection.getBalance(pg.wallet.publicKey);
console.log(`Balance: ${balance / web3.LAMPORTS_PER_SOL} SOL\n`);

const [tiendaPDA] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("tienda"), pg.wallet.publicKey.toBuffer()],
  pg.program.programId
);
console.log("PDA de la Tienda:", tiendaPDA.toBase58());

(async () => {
  // ===========================================================================
  // PASO 1 - CREATE: Inaugurar la Tienda
  // ===========================================================================
  console.log("\n-----------------------------------------");
  console.log("PASO 1 - Inaugurar la Tienda");
  console.log("-----------------------------------------");

  const cuentaExistente = await pg.connection.getAccountInfo(tiendaPDA);

  if (cuentaExistente === null) {
    const tx = await pg.program.methods
      .inaugurarTienda("Disco Vintage MX")
      .accounts({
        tienda: tiendaPDA,
        propietario: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    await esperar(tx);
    console.log("Tienda inaugurada. TX:", tx);
  } else {
    console.log("La tienda ya existe en la blockchain, reutilizando...");
  }

  let tienda = await pg.program.account.tienda.fetch(tiendaPDA);
  console.log(
    `   Tienda: "${tienda.nombre}" | Vinilos: ${tienda.catalogo.length}`
  );

  // ===========================================================================
  // PASO 2 - CREATE: Agregar Vinilos
  // ===========================================================================
  console.log("\n-----------------------------------------");
  console.log("PASO 2 - Agregar Vinilos al Catalogo");
  console.log("-----------------------------------------");

  const vinilos = [
    {
      artista: "Pink Floyd",
      album: "The Dark Side of the Moon",
      genero: "Rock Progresivo",
      precio: 85000,
      stock: 3,
    },
    {
      artista: "Miles Davis",
      album: "Kind of Blue",
      genero: "Jazz",
      precio: 72000,
      stock: 5,
    },
    {
      artista: "Led Zeppelin",
      album: "IV",
      genero: "Rock Clasico",
      precio: 68000,
      stock: 2,
    },
  ];

  for (const v of vinilos) {
    const yaExiste = tienda.catalogo.some(
      (c) => c.album === v.album && c.artista === v.artista
    );
    if (yaExiste) {
      console.log(`'${v.album}' ya esta en el catalogo, omitiendo.`);
      continue;
    }
    const tx = await pg.program.methods
      .agregarVinilo(
        v.artista,
        v.album,
        v.genero,
        new anchor.BN(v.precio),
        v.stock
      )
      .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
      .rpc();
    await esperar(tx);
    console.log(`'${v.album}' agregado. TX: ${tx}`);
  }

  tienda = await pg.program.account.tienda.fetch(tiendaPDA);
  console.log(`\n   Catalogo (${tienda.catalogo.length} vinilos):`);
  for (const v of tienda.catalogo) {
    console.log(
      `   [*] '${v.album}' - ${v.artista} | $${(v.precio / 100).toFixed(
        2
      )} MXN | Stock: ${v.stock} | ${v.estado}`
    );
  }

  // ===========================================================================
  // PASO 3 - READ: Ver catalogo on-chain
  // ===========================================================================
  console.log("\n-----------------------------------------");
  console.log("PASO 3 - READ: Ver Catalogo (log on-chain)");
  console.log("-----------------------------------------");

  const txRead = await pg.program.methods
    .verCatalogo()
    .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
    .rpc();
  await esperar(txRead);
  console.log("Catalogo registrado en el log de la TX:", txRead);
  console.log("(Busca esa TX en Solana Explorer para ver el detalle completo)");

  // ===========================================================================
  // PASO 4 - UPDATE: Actualizar un vinilo
  // ===========================================================================
  console.log("\n-----------------------------------------");
  console.log("PASO 4 - UPDATE: Actualizar 'Kind of Blue'");
  console.log("-----------------------------------------");

  const txUpdate = await pg.program.methods
    .actualizarVinilo("Kind of Blue", new anchor.BN(65000), 0, "Agotado")
    .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
    .rpc();
  await esperar(txUpdate);
  console.log("Vinilo actualizado. TX:", txUpdate);

  tienda = await pg.program.account.tienda.fetch(tiendaPDA);
  const vUpd = tienda.catalogo.find((v) => v.album === "Kind of Blue");
  console.log(
    `   Precio: $${(vUpd.precio / 100).toFixed(2)} | Stock: ${
      vUpd.stock
    } | Estado: ${vUpd.estado}`
  );

  // ===========================================================================
  // PASO 5 - DELETE: Retirar un vinilo
  // ===========================================================================
  console.log("\n-----------------------------------------");
  console.log("PASO 5 - DELETE: Retirar 'IV' de Led Zeppelin");
  console.log("-----------------------------------------");

  const txDelete = await pg.program.methods
    .retirarVinilo("IV")
    .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
    .rpc();
  await esperar(txDelete);
  console.log("Vinilo retirado. TX:", txDelete);

  tienda = await pg.program.account.tienda.fetch(tiendaPDA);
  console.log(`\n   Vinilos restantes: ${tienda.catalogo.length}`);
  for (const v of tienda.catalogo) {
    console.log(
      `   [*] '${v.album}' - ${v.artista} | $${(v.precio / 100).toFixed(2)} | ${
        v.estado
      }`
    );
  }

  console.log("\n-----------------------------------------");
  console.log("Ciclo CRUD del Vinilo Store completado.");
  console.log("-----------------------------------------\n");
})();
