import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { environment } from 'environments/environment.development';

@Injectable({
  providedIn: 'root',
})
export class FileStoragesService {
  private http = inject(HttpClient);
  private API_URL = environment.API_URL + 'file-storages';

  getAllFileStorages(page: number, pageSize: number, search?: string) {
    const params = new URLSearchParams();
    params.append('page', page.toString());
    params.append('pageSize', pageSize.toString());
    if (search) {
      params.append('search', search);
    }
    return this.http.get<any[]>(`${this.API_URL}?${params.toString()}`);
  }
  
  getFileStorageById(id: string) {
    return this.http.get<any>(`${this.API_URL}/${id}`);
  }

  createFileStorage(data: any) {
    return this.http.post<any>(`${this.API_URL}`, data);
  }

  updateFileStorage(id: string, data: any) {
    return this.http.put<any>(`${this.API_URL}/${id}`, data);
  }

  deleteFileStorage(id: string) {
    return this.http.delete<any>(`${this.API_URL}/${id}`);
  }
}
