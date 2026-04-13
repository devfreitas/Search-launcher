use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, MessageBoxW, MB_ICONERROR, MB_OK, MSG, WM_HOTKEY,
};
use windows::core::w;
use crossbeam_channel::Sender;

pub fn listen_for_hotkey(tx: Sender<()>) {
    unsafe {
        let result = RegisterHotKey(None, 1, MOD_ALT, 0x57);

        if result.is_ok() {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_HOTKEY {
                    println!("Apanhei o Alt+W! A enviar sinal para a UI...");
                    let _ = tx.send(());
                }
            }
        } else {
            MessageBoxW(
                None,
                w!("Não foi possível registar o atalho de teclado! A tecla já está a ser usada por outro programa ou pelo sistema."),
                w!("Erro no Launcher"),
                MB_ICONERROR | MB_OK,
            );
        }
    }
}

