import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { environment } from 'environments/environment';

@Injectable({
  providedIn: 'root',
})
export class DomainsService {
  private http = inject(HttpClient);
  private API_URL = environment.API_URL + 'domains';

  getAllDomains(page: number, pageSize: number, search?: string, port?: number) {
    const params = new URLSearchParams();
    params.append('page', page.toString());
    params.append('pageSize', pageSize.toString());
    if (search) {
      params.append('search', search);
    }
    if (port) {
      params.append('port', port.toString());
    }
    return this.http.get<any[]>(`${this.API_URL}?${params.toString()}`);
  }

  getDomainById(id: string) {
    return this.http.get<any>(`${this.API_URL}/${id}`);
  }

  createDomain(data: any) {
    return this.http.post<any>(`${this.API_URL}`, data);
  }

  updateDomain(id: string, data: any) {
    return this.http.put<any>(`${this.API_URL}/${id}`, data);
  }

  deleteDomain(id: string) {
    return this.http.delete<any>(`${this.API_URL}/${id}`);
  }
}
