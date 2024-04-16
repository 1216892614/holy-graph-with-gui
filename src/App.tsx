import { range } from "ramda";
import { useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

function App() {
    const diceInput = useRef<HTMLInputElement>(null);

    const [inputDice, setInputDice] = useState("");
    const isInputDiceFine = useMemo(
        () =>
            inputDice.split("").every((s) => {
                const num = Number.parseInt(s);
                return !Number.isNaN(num) && num <= 6 && num > 0;
            }),
        [inputDice]
    );
    const inputDiceArr = useMemo(() => inputDice.split(""), [inputDice]);

    const [spellLv, setSpellLv] = useState(1);

    const [result, setResult] = useState<string | null>(null);

    async function compute() {
        if (inputDiceArr.length <= 0) return setResult("Need dices input.");

        setResult("computing...");

        setResult(
            await invoke("compute", {
                inputD6: inputDiceArr.join("|"),
                inputLv: spellLv.toString(),
            })
        );
    }

    return (
        <main className="h-screen w-screen py-10 flex flex-col items-center justify-center px-2">
            <form
                onSubmit={(evt) => {
                    evt.preventDefault();
                    compute();
                }}
            >
                <div
                    onClick={() => diceInput.current?.focus()}
                    className="w-screen px-5 flex flex-wrap items-center justify-center [&>*]:m-2"
                >
                    {range(0, Math.max(inputDice.length, 4)).map((i) => (
                        <div
                            className={`card w-20 h-20 border-solid border-2 ${
                                isInputDiceFine
                                    ? "border-primary"
                                    : "border-error"
                            } shadow-xl`}
                        >
                            <div
                                className={`card-body ${
                                    isInputDiceFine
                                        ? "text-primary"
                                        : "text-error"
                                } text-5xl flex items-center justify-center leading-3`}
                            >
                                {inputDiceArr[i] ??
                                    (inputDiceArr.length <= 0
                                        ? ["D", "I", "C", "E"][i]
                                        : "")}
                            </div>
                        </div>
                    ))}
                </div>

                <input
                    ref={diceInput}
                    onChange={(evt) => setInputDice(evt.target.value)}
                    className="h-0 w-0"
                />

                <div className="card bg-base-200 w-96 m-auto shadow-xl">
                    <div className="card-body">
                        {isInputDiceFine ? (
                            <p>
                                <span className="text-primary">
                                    {inputDiceArr.length + " "}
                                </span>
                                D6s Total, spell lv
                                <span className="text-primary">
                                    {spellLv + 1}
                                </span>
                            </p>
                        ) : (
                            <p className="text-error">Input D6 results plz...</p>
                        )}

                        <input
                            type="range"
                            min={0}
                            max="8"
                            value={spellLv}
                            onChange={(evt) =>
                                setSpellLv(Number.parseInt(evt.target.value))
                            }
                            className="range"
                            step="1"
                        />
                        <div className="w-full flex justify-between text-xs px-2 mb-2">
                            {range(0, 9).map(() => (
                                <span>|</span>
                            ))}
                        </div>

                        <div className="card-actions justify-end">
                            <button type="submit" className="btn btn-primary">
                                Compute
                            </button>
                        </div>
                    </div>
                </div>
            </form>

            <div className="card bg-primary mt-5 w-96 m-auto shadow-xl">
                <div className="card-body shadow-inner text-center text-primary-content">
                    {result ?? "Waiting for compute"}
                </div>
            </div>
        </main>
    );
}

export default App;
