// Routes each screen to its view.
import { useApp } from './store'
import { Class } from './screens/Class'
import { Home } from './screens/Home'
import { Intro } from './screens/Intro'
import { Lab } from './screens/Lab'
import { Lesson } from './screens/Lesson'
import { Profiles } from './screens/Profiles'
import { Settings } from './screens/Settings'
import { Setup } from './screens/Setup'

export function Screens() {
  const { state } = useApp()
  switch (state.screen) {
    case 'intro': return <Intro />
    case 'setup': return <Setup />
    case 'profiles': return <Profiles />
    case 'settings': return <Settings />
    case 'home': return <Home />
    case 'class': return <Class />
    case 'lesson': return <Lesson />
    case 'lab': return <Lab />
  }
}
