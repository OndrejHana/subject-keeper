import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog"

export default function Welcome() {
    async function onClick() {
        const file = await open({
            directory: true,
            multiple: false,
        });

        if (!file) {
            return
        }

        await invoke("init_main_window", { homeDir: file });
    }

    return (
        <div className="w-screen h-screen p-6 flex flex-col justify-end">
            <button className="w-full bg-primary text-primary-foreground font-bold rounded shadow-lg" onMouseDown={onClick}>
                Choose a home directory
            </button>
        </div>
    )
}
