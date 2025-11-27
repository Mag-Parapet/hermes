import { AsyncPipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router, RouterLink } from '@angular/router';
import { Store } from '@ngrx/store';
import { toggleSidenav } from 'app/state/sidenav/sidenav.actions';
import { selectSidenavState } from 'app/state/sidenav/sidenav.reduce';
import { filter } from 'rxjs';

@Component({
  selector: 'app-header',
  imports: [RouterLink, AsyncPipe],
  templateUrl: './header.html',
  styleUrl: './header.css',
})
export class Header {
  store = inject(Store)
  route = inject(ActivatedRoute)
  router = inject(Router)

  collapsed$ = this.store.select(selectSidenavState)

  paths: any[] = [
    { title: 'Domains', route: '/domains' },
    { title: 'Add', route: '/domains/add' },
    { title: '', route: '/domains/:domainId' },
    { title: 'Edit', route: '/domains/:domainId/edit' },
    { title: 'File Storages', route: '/file-storages' },
    { title: 'Add', route: '/file-storages/add' },
    { title: '', route: '/file-storages/:storageId' },
    { title: 'Files', route: '/file-storages/:storageId/files' },
    { title: 'Edit', route: '/file-storages/:storageId/edit' },
  ];

  path = signal<any[]>([]);

  constructor() {
    this.router.events
      .pipe(filter(event => event instanceof NavigationEnd))
      .subscribe((res: NavigationEnd) => {
        const urlSegments = res.urlAfterRedirects.split('/').filter(Boolean);
        let currentUrl = '';
        this.path.set([
          { title: 'Home', route: '/' }
        ]);

        for (let i = 0; i < urlSegments.length; i++) {
          currentUrl += '/' + urlSegments[i];

          const match = this.paths.find(p => {
            const patternParts = p.route.split('/').filter(Boolean);
            const currentParts = currentUrl.split('/').filter(Boolean);

            if (patternParts.length !== currentParts.length) return false;

            return patternParts.every((part: any, idx: any) =>
              part.startsWith(':') || part === currentParts[idx]
            );
          });

          if (match) {
            const finalRoute = this.fillParams(match.route, urlSegments);
            let param = match.route.split('/').pop();
            if (history.state[param]) {
              this.path.set([...this.path(), { title: history.state[param], route: finalRoute }])
            } else {
              this.path.set([...this.path(), { title: match.title, route: finalRoute }]);
            }
          }
        }

      });
  }

  private fillParams(route: string, segments: string[]): string {
    const parts = route.split('/').filter(Boolean);
    let segIndex = 0;
    return (
      '/' +
      parts
        .map(part => {
          if (part.startsWith(':')) {
            return segments[segIndex++];
          }
          segIndex++;
          return part;
        })
        .join('/')
    );
  }

  onToggleSidenav() {
    this.store.dispatch(toggleSidenav());
  }
}
