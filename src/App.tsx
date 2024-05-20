import { useEffect, useState } from "react";
import { Button } from "./components/ui/button";
import { FolderOpen, Home as HomeIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function Home({ setRoute }: any) {
  const [rows, setRows] = useState([]);
  const [subjectName, setSubjectName] = useState("");

  useEffect(() => {
    (async () => {
      const subjects = await invoke("get_subjects");
      const newSubjects = subjects.map((subject: any) => ({ name: subject }));
      setRows(newSubjects);
    })();
  }, []);

  return (
    <div className="w-full h-full p-2 flex flex-col gap-2">
      <form className="flex items-center gap-2" onSubmit={async (formEvent) => {
        formEvent.preventDefault();
        await invoke("add_subject", { name: subjectName });
        setRows([...rows, { name: subjectName }]);
        setSubjectName("");
      }} >

        <input
          type="text"
          placeholder="Subject Name"
          className="p-2 border border-gray-300 rounded-md"
          value={subjectName}
          onChange={(e) => setSubjectName(e.target.value)}
        />
        <Button >Add Subject</Button>
      </form>
      <div className="w-full flex flex-col items-center gap-1">
        {rows.map((row: any, index: number) => (
          <div key={index} className="w-full p-2 border border-gray-300 rounded-md cursor-pointer hover:bg-muted" onClick={() => {
            setRoute({ name: "subject", data: row });
          }}>
            {row.name}
          </div>
        ))}
      </div>
      </div>
  );
}

function Subject({ data }: any) {
  const [files, setFiles] = useState([]);

  useEffect(() => {
    listen("fileDropped", async (event) => {
      const file = event.payload;
      const name = file.split("/").pop();
      await invoke("add_file", { subject: data.name, path:file });
      setFiles([...files, {name: name, path: file}]);
    });
  }, [data]);

  useEffect(() => {
    (async () => {
      const readFiles: Array<[name, path]> = await invoke("get_subject", { subject: data.name });
      const newFiles = readFiles.map(([name, path]: [name, path]) => ({ name: name, path: path }));
      setFiles(newFiles);
    })();
  }, [data]);

  return <div className="p-2 flex flex-col gap-2">
    <div className="flex border-b">
      <h2 className="w-full text-xl font-bold">{data.name}</h2>
      <Button variant="ghost" onClick={async () => {
        await invoke("open_folder", { subject: data.name });
      }}>
        <FolderOpen />
      </Button> 
    </div>
    <div>
      {files.map((file: any, index: number) => (
        <div key={index} className="w-full p-1 hover:bg-secondary rounded cursor-pointer" onClick={ async () => {
          await invoke("open_file", { path: file.path });
        }}>{file.name}</div>
      ))}
    </div>
  </div>;
}

function App() {
  const [route, setRoute] = useState({name: "home"});

  return (
    <div className="w-full h-full flex flex-col">
      <div className="w-full p-2">
        <Button variant="ghost" onClick={() => setRoute({name: "home"})}>
          <HomeIcon size={16} />
        </Button>
      </div>
      {route.name === "home" && <Home setRoute={setRoute} />}
      {route.name === "subject" && <Subject data={route.data} />}
    </div>
  );
}

export default App;
