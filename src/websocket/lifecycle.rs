use std::sync::Arc;

use anyhow::{anyhow, bail, Ok, Result};
use rand::{rngs, SeedableRng};
use socketioxide::extract::{Data, SocketRef, State};
use tracing::{error, info};

use crate::{
    game::digsites::DigSite,
    geometry::{Point, Size},
};

use super::state::{Connection, Parties};

pub fn on_connect(socket: SocketRef, parties: State<Parties>) {
    let Some(query) = socket.extensions.get::<Connection>() else {
        let res = socket.disconnect();
        if let Result::Err(err) = res {
            error!("Socket Create Error: {}", err);
        }
        return;
    };
    let conn = query.clone();

    info!(
        "Socket.IO connected: {:?} {:?} {:?}",
        socket.ns(),
        socket.id,
        query.room()
    );

    socket.on_disconnect(on_disconnect);
    socket.on(
        "move",
        |s: SocketRef, d: Data<String>, parties: State<Parties>| {
            let conn = s.extensions.get::<Connection>().unwrap().clone();
            let res = move_player(s.clone(), conn, parties, d.0.clone());
            if let Result::Err(err) = res {
                error!("Move Error: {}", err);
                // Attempt to disconnect the socket on failure
                let _ = s.clone().disconnect();
            };
        },
    );
    socket.on("game", |s: SocketRef, parties: State<Parties>| {
        let conn = s.extensions.get::<Connection>().unwrap().clone();
        let res = new_game(s.clone(), conn, parties);
        if let Result::Err(err) = res {
            error!("Move Error: {}", err);
            // Attempt to disconnect the socket on failure
            let _ = s.clone().disconnect();
        };
    });

    let res = init_user(socket.clone(), conn, parties);
    if let Result::Err(err) = res {
        error!("Socket Create Error: {}", err);
        // Attempt to disconnect the socket on failure
        let _ = socket.clone().disconnect();
    };
}

fn move_player(
    socket: SocketRef,
    conn: Connection,
    parties: State<Parties>,
    data: String,
) -> Result<()> {
    let instance = conn.room();
    let party = parties
        .get(instance.clone())
        .ok_or(anyhow!("party not initialized"))?;
    let digsite = Arc::clone(&party.game);
    let mut party_game = digsite
        .lock()
        .map_err(|_| anyhow!("Failed to lock digsite"))?; // Handle lock error
    let game = party_game.as_mut().ok_or(anyhow!("game not initialized"))?;

    let offset = match data.as_str() {
        "up" => Point { x: 0, y: -1 },
        "down" => Point { x: 0, y: 1 },
        "left" => Point { x: -1, y: 0 },
        "right" => Point { x: 1, y: 0 },
        _ => bail!("invalid move"),
    };

    game.move_player(conn.user.id.clone(), offset)?;

    socket
        .within(instance.clone())
        .emit("game", game.output())?;

    Ok(())
}

fn new_game(socket: SocketRef, conn: Connection, parties: State<Parties>) -> Result<()> {
    let instance = conn.room();
    let party = parties
        .get(instance.clone())
        .ok_or(anyhow!("party not initialized"))?;
    let digsite = Arc::clone(&party.game);
    let mut party_game = digsite
        .lock()
        .map_err(|_| anyhow!("Failed to lock digsite"))?; // Handle lock error

    let mut rng = rngs::StdRng::from_entropy();
    party_game.replace(DigSite::generate(
        &mut rng,
        Size { x: 10, y: 10 },
        15,
        Point { x: 5, y: 5 },
    )?);

    let game = party_game.as_mut().ok_or(anyhow!("game not initialized"))?;
    party.players.iter().for_each(|p| {
        game.add_player(p.clone()).unwrap();
    });

    socket
        .within(instance.clone())
        .emit("game", game.output())?;

    Ok(())
}

fn init_user(socket: SocketRef, conn: Connection, parties: State<Parties>) -> Result<()> {
    let instance = conn.room();

    socket.join(instance.clone())?;
    parties.ensure_party(instance.clone(), conn.user.id.clone());

    let party = parties
        .get(instance.clone())
        .ok_or(anyhow!("party not initialized"))?;

    info!("Party {} now {} large", party.id, party.players.len());

    socket
        .within(instance.clone())
        .emit("party", vec![party.players.iter().collect::<Vec<_>>()])?;

    let digsite = Arc::clone(&party.game);
    let mut party_game = digsite
        .lock()
        .map_err(|_| anyhow!("Failed to lock digsite"))?; // Handle lock error

    if party_game.is_none() {
        let mut rng = rngs::StdRng::from_entropy();
        party_game.replace(DigSite::generate(
            &mut rng,
            Size { x: 10, y: 10 },
            15,
            Point { x: 5, y: 5 },
        )?);
    }

    let game = party_game.as_mut().ok_or(anyhow!("game not initialized"))?;
    game.add_player(conn.user.id)?;

    socket
        .within(instance.clone())
        .emit("game", game.output())?;

    Ok(())
}

fn delete_user(socket: &SocketRef, conn: &Connection, parties: &Parties) -> Result<()> {
    let instance = conn.room();
    socket.leave(instance.clone())?;

    let was_deleted = parties.on_player_left(instance.clone(), conn.user.id.clone());
    if was_deleted {
        let err = socket.to(instance.clone()).disconnect();
        if err.is_err() {
            bail!("failed to disconnect sockets");
        }
        return Ok(());
    }

    let party = parties
        .get(instance.clone())
        .ok_or(anyhow!("party not initialized"))?;

    info!("Party {} now {} large", party.id, party.players.len());

    socket
        .within(instance.clone())
        .emit("party", vec![party.players.iter().collect::<Vec<_>>()])?;

    Ok(())
}

fn on_disconnect(socket: SocketRef, parties: State<Parties>) {
    if let Some(query) = socket.extensions.get::<Connection>() {
        info!(
            "Socket.IO disconnecting: {:?} {:?} {:?}",
            socket.ns(),
            socket.id,
            query.room()
        );

        let res = delete_user(&socket, &query, &parties);
        if let Result::Err(err) = res {
            error!("Socket Delete Error: {}", err);
        }
    }
}
