import { LayoutGrid, LayoutList, PanelRightClose, Search } from "lucide-react";
import { Card } from "./components/ui/card";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "./components/ui/resizable";
import { Button } from "./components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "./components/ui/tabs";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Subject = {
    id: string;
    name: string;
    path: string;
    icon: string | null;
    exists: boolean;
}

type Entry = {
    id: string;
    name: string;
    path: string;
    parent: string;
    exists: boolean;
}

async function getAllSubjects() {
    return invoke<Subject[]>("get_all_subjects");
}
async function getAllEntries() {
    return invoke<Entry[]>("get_all_entries");
}

async function openFile(path: string) {
    return invoke("open_file", {path});
}

function SubjectView({subjects, entries, setEntryPreview}: {subjects: Subject[], entries: Entry[], setEntryPreview: any}) {
    const [selectedSubject, setSelectedSubject] = useState<Subject | null>(null)

    if (selectedSubject) {
        const selectedSubjectEntries = entries.filter((e) => e.parent === selectedSubject.id)
        return <SubjectDetail subject={selectedSubject} entries={selectedSubjectEntries} setEntryPreview={setEntryPreview} />
    }

    return (
        <>
            {subjects.map((s) => (<div key={s.id} className="w-full flex justify-between" onMouseDown={() => setSelectedSubject(s)}>{s.name}</div>))}
        </>
    );
}

function EntriesView({entries, setEntryPreview}: {entries: Entry[], setEntryPreview: any}) {
    return <>
        {entries.map(e => <div key={e.id} className={`w-full flex justify-between ${!e.exists ? "text-muted-foreground" :""}`}onMouseDown={() => setEntryPreview(e)}onDoubleClick={() => openFile(e.path)}>{e.name}</div>)}
    </>
}

function SubjectDetail({entries, setEntryPreview}: {subject:Subject, entries: Entry[], setEntryPreview: any}) {
    return <div>
        {entries.map(e => <div key={e.id} className={`w-full flex justify-between ${!e.exists ? "text-muted-foreground" :""}`} onMouseDown={() => setEntryPreview(e)} onDoubleClick={() => openFile(e.path)}>{e.name}</div>)}
    </div>
}

function EntryPreview({entry} : {entry: Entry}) {
    return <div>
        <h2 className="font-bold text-xl">{entry.name}</h2>
        <p>{entry.path}</p>
    </div>
}

function App() {
    const [subjects, setSubjects] = useState<Subject[]>(new Array());
    const [entries, setEntries] = useState<Entry[]>(new Array());
    const [entryPreview, setEntryPreview] = useState<Entry | null>(null);
    const [view, setView] = useState("subjects");

    useEffect(() => {
        async function getAll() {
            const subjects = await getAllSubjects();
            setSubjects(subjects);
            const entries = await getAllEntries();
            setEntries(entries);
            console.log("run")
        }

        window.addEventListener("focus", getAll)

        getAll();

        return () => window.removeEventListener("focus", getAll)
    },[]);

    return (
        <ResizablePanelGroup direction="horizontal">
            <ResizablePanel defaultSize={15}>
                <h2>tags</h2>
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel className="flex flex-col">
                <nav className="flex justify-between items-center border-b border-muted px-2">
                    <div className="grow">
                        tabs
                    </div>
                    <div className="flex gap-1 p-1">
                        <Tabs defaultValue="list">
                            <TabsList>
                                <TabsTrigger value="list" ><LayoutList className="w-4 h-4"/></TabsTrigger>
                                <TabsTrigger value="grid"><LayoutGrid className="w-4 h-4"/></TabsTrigger>
                            </TabsList>
                        </Tabs>
                        <Button variant="ghost" className="px-3"><PanelRightClose className="w-4 h-4"/></Button>
                        <div className="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                            <Card className="flex items-center justify-between p-2 gap-2">
                                <Search className="w-4 h-4" />
                                <input type="text" placeholder="search for files..." className="grow bg-background outline-none" />
                            </Card>
                        </div>
                    </div>
                </nav>
                <div className="grow">
                    <div className="w-full bg-muted flex items-center justify-between px-2">
                        <h2>View</h2>
                        <Tabs  value={view} onValueChange={(val) => setView(val)}>
                        <TabsList>
                            <TabsTrigger value="subjects" >Subjects</TabsTrigger>
                            <TabsTrigger value="entries">Entries</TabsTrigger>
                        </TabsList>
                        </Tabs>
                    </div>
                <ResizablePanelGroup direction="horizontal">
                    <ResizablePanel className="flex flex-col">
                        {view === "subjects" ? <SubjectView subjects={subjects} entries={entries} setEntryPreview={setEntryPreview}/> : null}
                        {view === "entries" ? <EntriesView entries={entries} setEntryPreview={setEntryPreview}/> : null}
                    </ResizablePanel>
                    <ResizableHandle />
                    <ResizablePanel>
                        {entryPreview ? <EntryPreview entry={entryPreview}/> : null}
                    </ResizablePanel>
                </ResizablePanelGroup>
                </div>
            </ResizablePanel>
        </ResizablePanelGroup>
    )
}

export default App;
