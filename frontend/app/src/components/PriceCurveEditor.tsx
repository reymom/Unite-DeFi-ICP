import type React from "react"
import { Plus, Minus } from "lucide-react"

interface PricePoint {
    time_offset_secs: bigint
    price_multiplier: number
}

interface PriceCurveEditorProps {
    curve: PricePoint[]
    onChange: (curve: PricePoint[]) => void
}

const PriceCurveEditor: React.FC<PriceCurveEditorProps> = ({ curve, onChange }) => {
    const addPoint = () => {
        const newPoint: PricePoint = {
            time_offset_secs: BigInt(curve.length * 300),
            price_multiplier: 0.9,
        }
        onChange([...curve, newPoint])
    }

    const removePoint = (index: number) => {
        if (curve.length > 2) {
            onChange(curve.filter((_, i) => i !== index))
        }
    }

    const updatePoint = (index: number, field: keyof PricePoint, value: string) => {
        const updated = [...curve]
        if (field === "time_offset_secs") {
            updated[index] = { ...updated[index], [field]: BigInt(value) }
        } else {
            updated[index] = { ...updated[index], [field]: Number.parseFloat(value) }
        }
        onChange(updated)
    }

    return (
        <div className="price-curve-editor">
            <div className="curve-header">
                <h4>Dutch Auction Price Curve</h4>
                <button type="button" onClick={addPoint} className="add-point-btn">
                    <Plus size={16} />
                    Add Point
                </button>
            </div>

            <div className="curve-points">
                {curve.map((point, index) => (
                    <div key={index} className="price-point">
                        <div className="point-inputs">
                            <label>
                                Time (seconds)
                                <input
                                    type="number"
                                    value={point.time_offset_secs.toString()}
                                    onChange={(e) => updatePoint(index, "time_offset_secs", e.target.value)}
                                />
                            </label>
                            <label>
                                Price Multiplier
                                <input
                                    type="number"
                                    step="0.01"
                                    min="0"
                                    max="2"
                                    value={point.price_multiplier}
                                    onChange={(e) => updatePoint(index, "price_multiplier", e.target.value)}
                                />
                            </label>
                        </div>
                        {curve.length > 2 && (
                            <button type="button" onClick={() => removePoint(index)} className="remove-point-btn">
                                <Minus size={16} />
                            </button>
                        )}
                    </div>
                ))}
            </div>

            <div className="curve-preview">
                <p>
                    This creates a Dutch auction where the price starts at 100% and decreases over time to encourage faster
                    resolution.
                </p>
            </div>
        </div>
    )
}

export default PriceCurveEditor
