import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-shell'
import './App.css'

interface WikipediaSection {
  title: string
  summary: string
  paragraphs: string[]
  image_url?: string
  url: string
}

interface Definition {
  word: string
  part_of_speech?: string
  definition: string
  example?: string
}

interface DefinitionSection {
  source: string
  definitions: Definition[]
}

interface LookupResult {
  query: string
  content_type: 'Word' | 'Entity' | 'Mixed'
  error?: string
  sections: {
    definitions?: DefinitionSection[]
    wikipedia?: WikipediaSection
  }
}

function App() {
  const [query, setQuery] = useState('')
  const [result, setResult] = useState<LookupResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [activeTab, setActiveTab] = useState<'dictionary' | 'wikipedia'>('dictionary')

  useEffect(() => {
    // Get initial query from command-line arguments
    const loadInitialQuery = async () => {
      try {
        const args: string[] = await invoke('get_initial_query')
        if (args.length > 0) {
          const initialQuery = args.join(' ')
          console.log('[INFO] [touchdictionary] [gui] Initial query from args:', initialQuery)
          handleLookup(initialQuery)
        } else {
          // No query provided - maybe show a prompt?
          setLoading(false)
          console.log('[INFO] [touchdictionary] [gui] No query provided on startup')
        }
      } catch (error) {
        console.error('[ERROR] [touchdictionary] [gui] Failed to get initial query:', error)
      }
    }
    
    loadInitialQuery()
  }, [])

  const handleLookup = async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      return
    }

    setQuery(searchQuery)
    setLoading(true)
    setResult(null)

    try {
      const result: LookupResult = await invoke('run_lookup_command', { query: searchQuery })
      setResult(result)
      console.log('[INFO] [touchdictionary] [gui] Lookup completed for:', searchQuery)
    } catch (error) {
      console.error('[ERROR] [touchdictionary] [gui] Lookup failed:', error)
      setResult({
        query: searchQuery,
        contentType: 'Word',
        sections: {},
        error: error instanceof Error ? error.message : String(error)
      } as LookupResult)
    } finally {
      setLoading(false)
    }
  }

  const closeWindow = async () => {
    try {
      await invoke('close_window')
    } catch (error) {
      console.error('[ERROR] [touchdictionary] [gui] Failed to close window:', error)
    }
  }

  const openWikipediaLink = async (url: string) => {
    try {
      await invoke('open_url', { url })
    } catch (error) {
      console.error('[ERROR] [touchdictionary] [gui] Failed to open URL:', error)
    }
  }

  const hasDictionary = result?.sections.definitions && result.sections.definitions.length > 0
  const hasWikipedia = result?.sections.wikipedia != null

  // Auto-set tab if one type is missing
  useEffect(() => {
    if (hasDictionary && !hasWikipedia) {
      setActiveTab('dictionary')
    } else if (!hasDictionary && hasWikipedia) {
      setActiveTab('wikipedia')
    }
  }, [hasDictionary, hasWikipedia])

  const renderContent = () => {
    if (loading) {
      return null
    }

    if (result?.error) {
      return (
        <div className="error-content">
          <h3 className="error-title">Error</h3>
          <p className="error-message">{result.error}</p>
        </div>
      )
    }

    if (!result) {
      return null
    }

    // Render based on active tab
    return (
      <>
        {activeTab === 'dictionary' && hasDictionary && (
          <section className="definition-section">
            <h2 className="section-header">Dictionary</h2>
            <div className="definitions">
              {result.sections.definitions!.map((section: DefinitionSection, sectionIdx: number) => (
                <div key={sectionIdx} className="definition-source">
                  <div className="source-name">{section.source}</div>
                  {section.definitions.map((def: Definition, defIdx: number) => (
                    <div key={defIdx} className="definition-item">
                      {def.part_of_speech && (
                        <span className="part-of-speech">{def.part_of_speech}</span>
                      )}
                      <div className="definition-text">{def.definition}</div>
                      {def.example && (
                        <div className="example">"{def.example}"</div>
                      )}
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </section>
        )}

        {activeTab === 'wikipedia' && hasWikipedia && (
          <section className="wikipedia-section">
            <h2 className="section-header">Wikipedia</h2>
            <div className="wikipedia-content">
              {result.sections.wikipedia!.paragraphs.map((para, idx) => (
                <p key={idx} className="wikipedia-paragraph">
                  {para}
                </p>
              ))}
              <a 
                href="#" 
                onClick={(e) => {
                  e.preventDefault()
                  openWikipediaLink(result.sections.wikipedia!.url)
                }}
                className="wikipedia-link"
              >
                Read more on Wikipedia →
              </a>
            </div>
          </section>
        )}
      </>
    )
  }

  if (!result && !loading) {
    return (
      <div className="loading-container">
        <div className="loading-spinner"></div>
      </div>
    )
  }

  return (
    <div className="app">
      <div className="popup-card">
        {/* Top header with word and close button - draggable */}
        <div className="header" data-tauri-drag-region>
          <div className="word-title">
            <span className="word">{query}</span>
          </div>
          <button className="close-button" onClick={closeWindow} aria-label="Close">
            <span>&#215;</span>
          </button>
        </div>

        {/* Content area */}
        <div className="content">
          {loading && (
            <div className="loading-content">
              <div className="loading-spinner"></div>
              <p>Looking up definition...</p>
            </div>
          )}

          {!loading && renderContent()}
        </div>

        {/* Bottom actions */}
        {(hasDictionary || hasWikipedia) && (
          <div className="actions">
            <button 
              className={`action-button ${activeTab === 'dictionary' ? 'primary' : ''}`} 
              onClick={() => setActiveTab('dictionary')}
              disabled={!hasDictionary}
            >
              Dictionary
            </button>
            <button 
              className={`action-button ${activeTab === 'wikipedia' ? 'primary' : ''}`} 
              onClick={() => setActiveTab('wikipedia')}
              disabled={!hasWikipedia}
            >
              Wikipedia
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default App