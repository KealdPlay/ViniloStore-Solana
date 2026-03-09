// tests/anchor.test.ts
// =============================================================================
// VINILO STORE — Suite de Pruebas (CRUD completo)
// =============================================================================

describe("Test del Vinilo Store", () => {
  it("Ciclo CRUD completo: Create → Read → Update → Delete", async () => {

    const [tiendaPDA] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("tienda"), pg.wallet.publicKey.toBuffer()],
      pg.program.programId
    );

    console.log("\n==========================================");
    console.log("VINILO STORE — Pruebas de Integración");
    console.log("==========================================");
    console.log("Propietario:", pg.wallet.publicKey.toBase58());
    console.log("PDA Tienda: ", tiendaPDA.toBase58());

    // =========================================================================
    // C — CREATE: Inaugurar la tienda
    // =========================================================================
    console.log("\n--- [C] CREATE: Inaugurar Tienda ---");

    const cuentaExistente = await pg.connection.getAccountInfo(tiendaPDA);

    if (cuentaExistente === null) {
      const tx = await pg.program.methods
        .inaugurarTienda("Disco Vintage MX")
        .accounts({
          tienda:        tiendaPDA,
          propietario:   pg.wallet.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();
      console.log("Tienda inaugurada. TX:", tx);
    } else {
      console.log("ℹLa tienda ya existe en la blockchain, reutilizando...");
    }

    let cuenta = await pg.program.account.tienda.fetch(tiendaPDA);
    console.log(`   Nombre: ${cuenta.nombre} | Vinilos actuales: ${cuenta.catalogo.length}`);

    // =========================================================================
    // C — CREATE: Agregar Vinilos
    // =========================================================================
    console.log("\n--- [C] CREATE: Agregar Vinilos ---");
    const vinilos = [
      { artista: "Pink Floyd",   album: "The Dark Side of the Moon", genero: "Rock Progresivo", precio: 85000, stock: 3 },
      { artista: "Miles Davis",  album: "Kind of Blue",              genero: "Jazz",             precio: 72000, stock: 5 },
      { artista: "Led Zeppelin", album: "IV",                        genero: "Rock Clasico",     precio: 68000, stock: 2 },
    ];

    for (const v of vinilos) {
      const yaExiste = cuenta.catalogo.some(
        c => c.album === v.album && c.artista === v.artista
      );

      if (yaExiste) {
        console.log(`ℹ'${v.album}' ya está en el catálogo, omitiendo.`);
        continue;
      }

      const tx = await pg.program.methods
        .agregarVinilo(v.artista, v.album, v.genero, new BN(v.precio), v.stock)
        .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
        .rpc();
      console.log(`'${v.album}' agregado. TX: ${tx}`);
    }

    // =========================================================================
    // R — READ: Ver el catálogo on-chain vía instrucción
    // =========================================================================
    console.log("\n--- [R] READ: Ver Catálogo (on-chain log) ---");
    const txRead = await pg.program.methods
      .verCatalogo()
      .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
      .rpc();
    console.log("Catálogo impreso en el log de la TX:", txRead);
    console.log("   (Abre la TX en Solana Explorer para ver el detalle de cada vinilo)");

    cuenta = await pg.program.account.tienda.fetch(tiendaPDA);
    console.log(`\n   Catálogo (${cuenta.catalogo.length} vinilos):`);
    for (const v of cuenta.catalogo) {
      console.log(`   '${v.album}' — ${v.artista} | $${(v.precio / 100).toFixed(2)} | Stock: ${v.stock} | ${v.estado}`);
    }

    // =========================================================================
    // U — UPDATE: Actualizar un vinilo
    // =========================================================================
    console.log("\n--- [U] UPDATE: Actualizar 'Kind of Blue' ---");
    const txUpdate = await pg.program.methods
      .actualizarVinilo("Kind of Blue", new BN(65000), 2, "Agotado")
      .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
      .rpc();
    console.log("Actualización exitosa. TX:", txUpdate);

    cuenta = await pg.program.account.tienda.fetch(tiendaPDA);
    const vUpd = cuenta.catalogo.find(v => v.album === "Kind of Blue");
    console.log(`   → Precio: $${(vUpd.precio / 100).toFixed(2)} | Stock: ${vUpd.stock} | Estado: ${vUpd.estado}`);

    // =========================================================================
    // D — DELETE: Retirar un vinilo
    // =========================================================================
    console.log("\n--- [D] DELETE: Retirar 'IV' de Led Zeppelin ---");
    const txDelete = await pg.program.methods
      .retirarVinilo("IV")
      .accounts({ tienda: tiendaPDA, propietario: pg.wallet.publicKey })
      .rpc();
    console.log("Vinilo retirado. TX:", txDelete);

    const cuentaFinal = await pg.program.account.tienda.fetch(tiendaPDA);
    console.log(`   Vinilos restantes: ${cuentaFinal.catalogo.length}`);

    console.log("\n==========================================");
    console.log("CRUD completo verificado con éxito.");
    console.log("==========================================\n");
  });
});
